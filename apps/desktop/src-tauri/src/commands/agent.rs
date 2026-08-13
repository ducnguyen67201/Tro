use crate::{app_state::AppState, domain::error::internal, services::overlay};
use contracts::{AgentEvent, AgentState, AppError, ErrorCode};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationDecision {
    AllowOnce,
    Stop,
}

#[tauri::command]
pub fn start_agent(
    app: AppHandle,
    state: State<'_, AppState>,
    goal: String,
    source_frame_id: Option<String>,
) -> Result<(), AppError> {
    let _source_frame_id = source_frame_id;
    let goal = goal.trim();
    if goal.len() < 3 || goal.len() > 500 {
        return Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Mục tiêu agent phải rõ ràng và không quá 500 ký tự.",
            false,
        ));
    }
    let foreground = state.foreground.snapshot();
    if foreground.is_elevated {
        return Err(AppError::new(
            ErrorCode::ElevatedTargetUnsupported,
            "Tro không điều khiển ứng dụng chạy quyền quản trị.",
            false,
        ));
    }
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot.agent = snapshot.agent.transition(AgentEvent::Start)?;
        snapshot.status_vi = "Agent đang lập kế hoạch trong phạm vi đã xác nhận.".to_owned();
        snapshot.clone()
    };
    state.reset_cancellation();
    sync_cursor_companion(&app, &state, snapshot.agent);
    app.emit("assistant_state_changed", snapshot)
        .map_err(|_| internal("Không thể cập nhật trạng thái agent."))
}

#[tauri::command]
pub fn resolve_confirmation(
    app: AppHandle,
    state: State<'_, AppState>,
    confirmation_id: String,
    decision: ConfirmationDecision,
) -> Result<(), AppError> {
    let _id = Uuid::parse_str(&confirmation_id).map_err(|_| {
        AppError::new(
            ErrorCode::InvalidRequest,
            "Xác nhận không hợp lệ hoặc đã hết hạn.",
            false,
        )
    })?;
    if matches!(decision, ConfirmationDecision::Stop) {
        return emergency_stop(app, state);
    }
    Err(AppError::new(
        ErrorCode::ActionRequiresConfirmation,
        "Xác nhận chỉ hợp lệ khi khớp thao tác và cửa sổ hiện tại.",
        false,
    ))
}

#[tauri::command]
pub fn emergency_stop(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.cancellation().cancel();
    state.audio.stop();
    state.input.release_all()?;
    state
        .confirmation
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    overlay::show_all(&app);
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
        snapshot.status_vi = "Đã dừng an toàn.".to_owned();
        snapshot.clone()
    };
    sync_cursor_companion(&app, &state, snapshot.agent);
    app.emit("assistant_state_changed", snapshot)
        .map_err(|_| internal("Không thể cập nhật trạng thái dừng."))
}

fn sync_cursor_companion(app: &AppHandle, state: &State<'_, AppState>, agent: AgentState) {
    let result = if companion_should_follow(agent) {
        state.cursor_companion.follow(app)
    } else {
        state.cursor_companion.hide(app)
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

fn companion_should_follow(agent: AgentState) -> bool {
    matches!(
        agent,
        AgentState::Idle | AgentState::Completed | AgentState::Stopped | AgentState::Failed
    )
}

#[cfg(test)]
mod tests {
    use contracts::AgentState;

    use super::companion_should_follow;

    #[test]
    fn companion_follows_only_outside_an_active_task() {
        for state in [
            AgentState::Idle,
            AgentState::Completed,
            AgentState::Stopped,
            AgentState::Failed,
        ] {
            assert!(companion_should_follow(state));
        }
        for state in [
            AgentState::Planning,
            AgentState::AwaitingConfirmation,
            AgentState::Executing,
            AgentState::Observing,
        ] {
            assert!(!companion_should_follow(state));
        }
    }
}
