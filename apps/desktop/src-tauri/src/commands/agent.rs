use std::time::Duration;

use crate::{
    app_state::AppState,
    domain::{error::internal, session::AgentLimits},
    security::action_policy::{ActionContext, ActionPolicy},
    services::overlay,
};
use contracts::{
    ActionOutcome, ActionReceipt, AgentEvent, AgentState, AppError, ComputerAction,
    CoordinateMapper, ErrorCode, PhysicalPoint, PlannedComputerAction, RiskTier, ScreenFrame,
    ScreenFrameMeta,
};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

const CONFIRMATION_WINDOW: &str = "confirmation";
const CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationDecision {
    AllowOnce,
    Stop,
}

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
    .inner_size(560.0, 340.0)
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
    let _source_frame_id = source_frame_id;
    run_agent_goal(&app, &state, &goal).await
}

pub async fn run_agent_goal(app: &AppHandle, state: &AppState, goal: &str) -> Result<(), AppError> {
    let goal = goal.trim();
    validate_goal(goal)?;
    let foreground = state.foreground.snapshot();
    if foreground.is_elevated {
        return Err(AppError::new(
            ErrorCode::ElevatedTargetUnsupported,
            "Tro không điều khiển ứng dụng chạy quyền quản trị.",
            false,
        ));
    }
    prepare_agent(app, state)?;
    let result = run_loop(app, state, goal).await;
    if let Err(error) = &result {
        let current = state
            .snapshot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .agent;
        if current != AgentState::Stopped {
            let _failed = set_agent(app, state, AgentEvent::Fail, &error.message_vi);
        }
        let _return = state.cursor_companion.return_to_cursor(app);
    }
    result
}

async fn run_loop(app: &AppHandle, state: &AppState, goal: &str) -> Result<(), AppError> {
    let config = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let cancellation = state.cancellation();
    let mut limits = AgentLimits::default();
    let first_frame = capture_current(app, state).await?;
    let mut action_frame_meta = first_frame.meta.clone();
    let mut response = tokio::select! {
        () = cancellation.cancelled() => return Err(cancelled()),
        result = state.computer_use.create_run(&config, goal, first_frame) => result?,
    };
    let mut active_run = Some(response.run_id.clone());

    loop {
        if let Err(error) =
            limits.record_turn(u32::try_from(response.actions.len()).unwrap_or(u32::MAX))
        {
            return stop_remote_and_error(state, &config, &mut active_run, error).await;
        }
        if response.completed {
            let message_vi = response
                .message_vi
                .take()
                .unwrap_or_else(|| "Tro đã hoàn thành tác vụ.".to_owned());
            set_agent(app, state, AgentEvent::Complete, &message_vi)?;
            crate::services::speech::speak_best_effort(state.speech.clone(), message_vi).await;
            return Ok(());
        }
        if response.actions.len() != 1 {
            return stop_remote_and_error(
                state,
                &config,
                &mut active_run,
                AppError::new(
                    ErrorCode::ProviderProtocolError,
                    "Computer use phải trả về đúng một thao tác mỗi lượt.",
                    true,
                ),
            )
            .await;
        }

        let planned = response.actions.remove(0);
        let frame_meta = action_frame_meta.clone();
        if let Err(error) = authorize_action(app, state, &planned).await {
            return stop_remote_and_error(state, &config, &mut active_run, error).await;
        }
        if state
            .snapshot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .agent
            == AgentState::AwaitingConfirmation
        {
            set_agent(
                app,
                state,
                AgentEvent::Confirm,
                "Đang thực hiện thao tác đã cho phép…",
            )?;
        } else {
            set_agent(
                app,
                state,
                AgentEvent::ActionsReady,
                &planned.description_vi,
            )?;
        }

        if let Err(error) = travel_to_action(app, state, &planned.action, &frame_meta) {
            return stop_remote_and_error(state, &config, &mut active_run, error).await;
        }
        let input = state.input.clone();
        let action = planned.action.clone();
        let action_frame = frame_meta.clone();
        let action_cancellation = cancellation.clone();
        let execution = match tokio::task::spawn_blocking(move || {
            input.execute(&action, &action_frame, &action_cancellation)
        })
        .await
        {
            Ok(execution) => execution,
            Err(_) => {
                return stop_remote_and_error(
                    state,
                    &config,
                    &mut active_run,
                    internal("Computer use không thể hoàn tất thao tác nhập."),
                )
                .await;
            }
        };
        let outcome = match execution {
            Ok(()) => ActionOutcome::Executed,
            Err(error) => {
                return stop_remote_and_error(state, &config, &mut active_run, error).await;
            }
        };
        set_agent(app, state, AgentEvent::Executed, "Đang kiểm tra kết quả…")?;
        tokio::select! {
            () = cancellation.cancelled() => {
                return stop_remote_and_error(state, &config, &mut active_run, cancelled()).await;
            }
            () = tokio::time::sleep(Duration::from_millis(420)) => {}
        }
        let next_frame = match capture_current(app, state).await {
            Ok(frame) => frame,
            Err(error) => {
                return stop_remote_and_error(state, &config, &mut active_run, error).await;
            }
        };
        action_frame_meta = next_frame.meta.clone();
        set_agent(app, state, AgentEvent::Observed, "Đang xem màn hình mới…")?;
        let receipts = vec![ActionReceipt {
            action_index: 0,
            outcome,
            error_code: None,
        }];
        let next_turn = response.turn_number.saturating_add(1);
        let next_response = tokio::select! {
            () = cancellation.cancelled() => {
                return stop_remote_and_error(state, &config, &mut active_run, cancelled()).await;
            }
            result = state.computer_use.next_turn(
                &config,
                &response.run_id,
                next_turn,
                receipts,
                next_frame,
            ) => result,
        };
        response = match next_response {
            Ok(response) => response,
            Err(error) => {
                return stop_remote_and_error(state, &config, &mut active_run, error).await;
            }
        };
    }
}

async fn capture_current(app: &AppHandle, state: &AppState) -> Result<ScreenFrame, AppError> {
    overlay::hide_all(app);
    tokio::time::sleep(Duration::from_millis(34)).await;
    let cursor = app.cursor_position().ok().map(|position| PhysicalPoint {
        x: position.x.round() as i32,
        y: position.y.round() as i32,
    });
    let capture = state.capture.clone();
    let result = tokio::task::spawn_blocking(move || capture.capture_display_at(cursor)).await;
    overlay::show_all(app);
    match result {
        Ok(result) => result,
        Err(_) => Err(internal("Không thể chụp màn hình cho computer use.")),
    }
}

async fn authorize_action(
    app: &AppHandle,
    state: &AppState,
    planned: &PlannedComputerAction,
) -> Result<(), AppError> {
    let foreground = state.foreground.snapshot();
    let decision = ActionPolicy::evaluate(
        &planned.action,
        &ActionContext {
            explicit_session: true,
            goal_matches: true,
            foreground: &foreground,
            target: planned.target,
        },
    );
    match decision.tier {
        RiskTier::Low => Ok(()),
        RiskTier::Blocked => Err(AppError::new(
            ErrorCode::UnsupportedAction,
            decision.display_vi,
            false,
        )),
        RiskTier::Confirm => {
            set_agent(
                app,
                state,
                AgentEvent::ConfirmationRequired,
                "Tro đang chờ bạn xác nhận một thao tác.",
            )?;
            request_confirmation(app, state, planned, &foreground).await
        }
    }
}

async fn request_confirmation(
    app: &AppHandle,
    state: &AppState,
    planned: &PlannedComputerAction,
    foreground: &contracts::ForegroundContext,
) -> Result<(), AppError> {
    let request = state
        .confirmation
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .issue(&planned.action, foreground)?;
    let id = Uuid::parse_str(&request.confirmation_id)
        .map_err(|_| internal("Không thể tạo xác nhận computer use."))?;
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
    let allowed = tokio::select! {
        () = cancellation.cancelled() => false,
        result = tokio::time::timeout(CONFIRMATION_TIMEOUT, receiver) => {
            matches!(result, Ok(Ok(true)))
        }
    };
    let _hide = window.hide();
    if !allowed {
        return Err(cancelled());
    }
    let current = state.foreground.snapshot();
    let consumed = state
        .confirmation
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .consume(id, &planned.action, &current);
    if !consumed {
        return Err(AppError::new(
            ErrorCode::ActionRequiresConfirmation,
            "Xác nhận đã hết hạn hoặc cửa sổ đã thay đổi.",
            false,
        ));
    }
    Ok(())
}

fn travel_to_action(
    app: &AppHandle,
    state: &AppState,
    action: &ComputerAction,
    frame: &ScreenFrameMeta,
) -> Result<(), AppError> {
    let point = match action {
        ComputerAction::Move { point } | ComputerAction::Click { point, .. } => Some(*point),
        ComputerAction::Drag { from, .. } => Some(*from),
        ComputerAction::Scroll { .. }
        | ComputerAction::TypeText { .. }
        | ComputerAction::KeyPress { .. }
        | ComputerAction::Wait { .. }
        | ComputerAction::Capture => None,
    };
    if let Some(point) = point {
        state
            .cursor_companion
            .travel_to_validated_target(app, CoordinateMapper::to_physical(point, frame))
            .map_err(|_| internal("Tro chưa thể di chuyển đến thao tác."))?;
    }
    Ok(())
}

async fn stop_remote_and_error(
    state: &AppState,
    config: &crate::services::llm::LlmConfig,
    run_id: &mut Option<String>,
    error: AppError,
) -> Result<(), AppError> {
    if let Some(run_id) = run_id.take() {
        state.computer_use.stop_run(config, &run_id).await;
    }
    Err(error)
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
    {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(
            snapshot.agent,
            AgentState::Completed | AgentState::Stopped | AgentState::Failed
        ) {
            snapshot.agent = AgentState::Idle;
        }
    }
    set_agent(
        app,
        state,
        AgentEvent::Start,
        "Computer use đang xem màn hình và lập kế hoạch…",
    )?;
    sync_cursor_companion(app, state, AgentState::Planning);
    Ok(())
}

fn set_agent(
    app: &AppHandle,
    state: &AppState,
    event: AgentEvent,
    status: &str,
) -> Result<AgentState, AppError> {
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot.agent = snapshot.agent.transition(event)?;
        snapshot.status_vi = status.to_owned();
        snapshot.clone()
    };
    sync_cursor_companion(app, state, snapshot.agent);
    app.emit("assistant_state_changed", snapshot.clone())
        .map_err(|_| internal("Không thể cập nhật trạng thái computer use."))?;
    Ok(snapshot.agent)
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

#[tauri::command]
pub fn resolve_confirmation(
    app: AppHandle,
    state: State<'_, AppState>,
    confirmation_id: String,
    decision: ConfirmationDecision,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(&confirmation_id).map_err(|_| {
        AppError::new(
            ErrorCode::InvalidRequest,
            "Xác nhận không hợp lệ hoặc đã hết hạn.",
            false,
        )
    })?;
    let allowed = matches!(decision, ConfirmationDecision::AllowOnce);
    if !state.resolve_confirmation_waiter(id, allowed) {
        return Err(AppError::new(
            ErrorCode::ActionRequiresConfirmation,
            "Xác nhận không còn hiệu lực.",
            false,
        ));
    }
    if let Some(window) = app.get_webview_window(CONFIRMATION_WINDOW) {
        let _hide = window.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn emergency_stop(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.cancellation().cancel();
    state.audio.stop();
    state.speech.stop();
    state.cancel_confirmation_waiter();
    state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    state.input.release_all()?;
    state
        .confirmation
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    overlay::show_all(&app);
    if let Some(window) = app.get_webview_window(CONFIRMATION_WINDOW) {
        let _hide = window.hide();
    }
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot.assistant = contracts::AssistantState::Idle;
        snapshot.agent = if snapshot.agent == AgentState::Idle {
            AgentState::Idle
        } else {
            snapshot.agent.transition(AgentEvent::Stop)?
        };
        snapshot.capture_active = false;
        snapshot.status_vi = "Đã dừng computer use an toàn.".to_owned();
        snapshot.clone()
    };
    sync_cursor_companion(&app, &state, snapshot.agent);
    app.emit("assistant_state_changed", snapshot)
        .map_err(|_| internal("Không thể cập nhật trạng thái dừng."))
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
        AgentState::Idle | AgentState::Planning | AgentState::AwaitingConfirmation => {
            CompanionBehavior::Follow
        }
        AgentState::Executing | AgentState::Observing => CompanionBehavior::StayWithAction,
        AgentState::Completed | AgentState::Stopped | AgentState::Failed => {
            CompanionBehavior::ReturnToCursor
        }
    }
}

fn cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "Đã dừng computer use.", false)
}

#[cfg(test)]
mod tests {
    use contracts::AgentState;

    use super::{CompanionBehavior, companion_behavior};

    #[test]
    fn companion_waits_at_cursor_until_an_action_has_a_target() {
        for state in [
            AgentState::Idle,
            AgentState::Planning,
            AgentState::AwaitingConfirmation,
        ] {
            assert_eq!(companion_behavior(state), CompanionBehavior::Follow);
        }
        for state in [AgentState::Executing, AgentState::Observing] {
            assert_eq!(companion_behavior(state), CompanionBehavior::StayWithAction);
        }
        for state in [
            AgentState::Completed,
            AgentState::Stopped,
            AgentState::Failed,
        ] {
            assert_eq!(companion_behavior(state), CompanionBehavior::ReturnToCursor);
        }
    }
}
