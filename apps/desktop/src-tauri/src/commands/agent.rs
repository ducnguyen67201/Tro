use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use contracts::{AgentState, AppError, ApplicationRef, ErrorCode};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    domain::{confirmation::ConfirmationChoice, error::internal},
    services::{
        agent_runtime::{AgentRuntime, AppApprovalDecision, RuntimeResult, RuntimeUi},
        overlay,
        stabilizer::Stabilizer,
        user_activity::UserActivityBackend,
    },
};

const CONFIRMATION_WINDOW: &str = "confirmation";
const CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(30);

pub fn create_confirmation_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(CONFIRMATION_WINDOW).is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        CONFIRMATION_WINDOW,
        WebviewUrl::App("index.html".into()),
    )
    .title("Tro cần xác nhận")
    .inner_size(560.0, 390.0)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .always_on_top(true)
    .visible(false)
    .center()
    .build()?;
    Ok(())
}

#[tauri::command]
pub async fn start_agent(
    app: AppHandle,
    state: State<'_, AppState>,
    goal: String,
    source_frame_id: Option<String>,
) -> Result<(), AppError> {
    crate::commands::auth::require_authentication(&app, &state)?;
    let _source_frame_id = source_frame_id;
    run_agent_goal(&app, &state, &goal, None).await
}

#[tauri::command]
pub async fn start_agent_for_app(
    app: AppHandle,
    state: State<'_, AppState>,
    goal: String,
    app_id: String,
) -> Result<(), AppError> {
    crate::commands::auth::require_authentication(&app, &state)?;
    if app_id.is_empty() || app_id.len() > 200 {
        return Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Ứng dụng đã chọn không hợp lệ.",
            false,
        ));
    }
    run_agent_goal(&app, &state, &goal, Some(&app_id)).await
}

pub async fn run_agent_goal(
    app: &AppHandle,
    state: &AppState,
    goal: &str,
    requested_app_id: Option<&str>,
) -> Result<(), AppError> {
    let goal = goal.trim();
    validate_goal(goal)?;
    prepare_agent(app, state)?;
    let config = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let activity: Arc<dyn UserActivityBackend> = state.user_activity.clone();
    let runtime = AgentRuntime::new(
        config,
        state.applications.clone(),
        state.app_approvals.clone(),
        state.observer.clone(),
        state.computer_use.clone(),
        state.action_executor.clone(),
        Stabilizer::new(state.applications.clone(), state.observer.clone(), activity),
    );
    let ui = TauriRuntimeUi { app, state };
    let result = runtime
        .run_for_app(goal, requested_app_id, &ui, state.cancellation())
        .await;
    match result {
        Ok(RuntimeResult::Completed(message_vi)) => {
            crate::services::speech::speak_best_effort(state.speech.clone(), message_vi).await;
            Ok(())
        }
        Ok(RuntimeResult::NeedsUser {
            message_vi,
            choices,
            ..
        }) => {
            set_runtime_status(
                app,
                state,
                AgentState::NeedsUser,
                &message_vi,
                None,
                choices,
            );
            Ok(())
        }
        Ok(RuntimeResult::PausedByUser) => {
            set_runtime_status(
                app,
                state,
                AgentState::PausedByUser,
                "Bạn đã tiếp quản — Tro đã dừng. Hãy yêu cầu lại để bắt đầu từ quan sát mới.",
                None,
                Vec::new(),
            );
            Ok(())
        }
        Err(error) => {
            let next = if error.code == ErrorCode::Cancelled {
                AgentState::Stopped
            } else {
                AgentState::Failed
            };
            set_runtime_status(app, state, next, &error.message_vi, None, Vec::new());
            let _return = state.cursor_companion.return_to_cursor(app);
            if error.code == ErrorCode::AuthExpired {
                crate::commands::auth::handle_auth_error(app, state, &error);
            }
            Err(error)
        }
    }
}

struct TauriRuntimeUi<'a> {
    app: &'a AppHandle,
    state: &'a AppState,
}

#[async_trait]
impl RuntimeUi for TauriRuntimeUi<'_> {
    fn status(&self, state: AgentState, message_vi: &str, app: Option<&ApplicationRef>) {
        set_runtime_status(self.app, self.state, state, message_vi, app, Vec::new());
    }

    async fn approve_app(&self, app: &ApplicationRef) -> Result<AppApprovalDecision, AppError> {
        let request = self
            .state
            .confirmation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .issue_app_access(app)?;
        let id = parse_confirmation_id(&request.confirmation_id)?;
        let decision = show_and_wait(self.app, self.state, id, request).await?;
        let valid = self
            .state
            .confirmation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .consume_app(id, app);
        if !valid {
            return Err(expired_confirmation());
        }
        Ok(match decision {
            ConfirmationChoice::AllowOnce => AppApprovalDecision::AllowOnce,
            ConfirmationChoice::AlwaysAllow => AppApprovalDecision::AlwaysAllow,
            ConfirmationChoice::Stop => AppApprovalDecision::Stop,
        })
    }

    async fn confirm_action(
        &self,
        scope_id: Uuid,
        app: &ApplicationRef,
        observation: &crate::services::observation::Observation,
        planned: &contracts::PlannedComputerAction,
    ) -> Result<bool, AppError> {
        let request = self
            .state
            .confirmation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .issue_action(scope_id, planned, &observation.metadata.binding, app)?;
        let id = parse_confirmation_id(&request.confirmation_id)?;
        let decision = show_and_wait(self.app, self.state, id, request).await?;
        if decision != ConfirmationChoice::AllowOnce {
            return Ok(false);
        }
        let valid = self
            .state
            .confirmation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .consume_action(id, scope_id, planned, &observation.metadata.binding);
        if !valid {
            return Err(expired_confirmation());
        }
        Ok(true)
    }
}

async fn show_and_wait(
    app: &AppHandle,
    state: &AppState,
    id: Uuid,
    request: crate::domain::confirmation::ConfirmationRequest,
) -> Result<ConfirmationChoice, AppError> {
    let receiver = state.wait_for_confirmation(id);
    let window = app
        .get_webview_window(CONFIRMATION_WINDOW)
        .ok_or_else(|| internal("Không tìm thấy cửa sổ xác nhận."))?;
    window
        .show()
        .and_then(|()| window.set_focus())
        .map_err(|_| internal("Không thể hiển thị xác nhận."))?;
    window
        .emit("confirmation_requested", request)
        .map_err(|_| internal("Không thể gửi yêu cầu xác nhận."))?;
    let cancellation = state.cancellation();
    let decision = tokio::select! {
        () = cancellation.cancelled() => ConfirmationChoice::Stop,
        result = tokio::time::timeout(CONFIRMATION_TIMEOUT, receiver) => {
            result.ok().and_then(Result::ok).unwrap_or(ConfirmationChoice::Stop)
        }
    };
    let _hide = window.hide();
    Ok(decision)
}

fn parse_confirmation_id(value: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| internal("Không thể tạo xác nhận computer use."))
}

fn expired_confirmation() -> AppError {
    AppError::new(
        ErrorCode::ActionRequiresConfirmation,
        "Xác nhận đã hết hạn hoặc cửa sổ đã thay đổi.",
        false,
    )
}

#[tauri::command]
pub fn resolve_confirmation(
    app: AppHandle,
    state: State<'_, AppState>,
    confirmation_id: String,
    decision: ConfirmationChoice,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(&confirmation_id).map_err(|_| {
        AppError::new(
            ErrorCode::InvalidRequest,
            "Xác nhận không hợp lệ hoặc đã hết hạn.",
            false,
        )
    })?;
    if !state.resolve_confirmation_waiter(id, decision) {
        return Err(expired_confirmation());
    }
    if let Some(window) = app.get_webview_window(CONFIRMATION_WINDOW) {
        let _hide = window.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn emergency_stop(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    emergency_stop_with_state(&app, &state)
}

pub(crate) fn emergency_stop_with_state(app: &AppHandle, state: &AppState) -> Result<(), AppError> {
    state.cancellation().cancel();
    state.audio.stop();
    state.speech.stop();
    state.cancel_confirmation_waiter();
    state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let release_error = state.action_executor.release_all().err();
    state
        .confirmation
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    *state
        .active_app_id
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    overlay::show_all(app);
    if let Some(window) = app.get_webview_window(CONFIRMATION_WINDOW) {
        let _hide = window.hide();
    }
    set_runtime_status(
        app,
        state,
        AgentState::Stopped,
        "Đã dừng computer use an toàn.",
        None,
        Vec::new(),
    );
    if let Some(error) = release_error {
        return Err(error);
    }
    Ok(())
}

fn prepare_agent(app: &AppHandle, state: &AppState) -> Result<(), AppError> {
    state.reset_cancellation();
    state.speech.stop();
    state
        .confirmation
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    state.cancel_confirmation_waiter();
    set_runtime_status(
        app,
        state,
        AgentState::ResolvingApp,
        "Computer use đang xác định ứng dụng…",
        None,
        Vec::new(),
    );
    Ok(())
}

fn set_runtime_status(
    app_handle: &AppHandle,
    state: &AppState,
    agent: AgentState,
    status_vi: &str,
    app: Option<&ApplicationRef>,
    choices: Vec<ApplicationRef>,
) {
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot.agent = agent;
        snapshot.status_vi = status_vi.to_owned();
        snapshot.scoped_app_name = app.map(|value| value.display_name.clone());
        snapshot.agent_choices = choices;
        snapshot.clone()
    };
    *state
        .active_app_id
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = app.map(|value| value.app_id.clone());
    sync_cursor_companion(app_handle, state, agent);
    if let Err(error) = app_handle.emit("assistant_state_changed", snapshot) {
        tracing::warn!(
            component = "agent_runtime",
            operation = "emit_status",
            error_code = "window_operation_failed",
            source = %error
        );
    }
}

fn sync_cursor_companion(app: &AppHandle, state: &AppState, agent: AgentState) {
    let result = match companion_behavior(agent) {
        CompanionBehavior::Follow => state.cursor_companion.follow(app),
        CompanionBehavior::StayWithAction => Ok(()),
        CompanionBehavior::ReturnToCursor => state.cursor_companion.return_to_cursor(app),
    };
    if let Err(error) = result {
        tracing::warn!(
            component = "cursor_companion",
            operation = "sync_agent_state",
            error_code = "window_operation_failed",
            source = %error
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompanionBehavior {
    Follow,
    StayWithAction,
    ReturnToCursor,
}

fn companion_behavior(agent: AgentState) -> CompanionBehavior {
    match agent {
        AgentState::Idle
        | AgentState::ResolvingApp
        | AgentState::AwaitingAppApproval
        | AgentState::ActivatingApp
        | AgentState::Planning
        | AgentState::Validating
        | AgentState::AwaitingConfirmation
        | AgentState::NeedsUser
        | AgentState::PausedByUser => CompanionBehavior::Follow,
        AgentState::Executing
        | AgentState::Stabilizing
        | AgentState::Observing
        | AgentState::StaleRecovery => CompanionBehavior::StayWithAction,
        AgentState::Completed | AgentState::Stopped | AgentState::Failed => {
            CompanionBehavior::ReturnToCursor
        }
    }
}

fn validate_goal(goal: &str) -> Result<(), AppError> {
    if goal.len() < 3 || goal.len() > 500 {
        return Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Mục tiêu computer use phải rõ ràng và không quá 500 ký tự.",
            false,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use contracts::AgentState;

    use super::{CompanionBehavior, companion_behavior};

    #[test]
    fn companion_stays_with_validation_and_execution() {
        for state in [
            AgentState::Executing,
            AgentState::Stabilizing,
            AgentState::Observing,
            AgentState::StaleRecovery,
        ] {
            assert_eq!(companion_behavior(state), CompanionBehavior::StayWithAction);
        }
    }
}
