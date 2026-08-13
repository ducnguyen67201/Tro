use crate::{app_state::AppState, domain::error::internal, services::overlay};
use contracts::{AppError, AssistantEvent, AssistantState, AssistantUiState};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_app_snapshot(state: State<'_, AppState>) -> AssistantUiState {
    state
        .snapshot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

#[tauri::command]
pub async fn start_assistant(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
) -> Result<(), AppError> {
    if source.len() > 32 {
        return Err(internal("Nguồn yêu cầu không hợp lệ."));
    }
    let _token = state.reset_cancellation();
    set_assistant(&app, &state, AssistantEvent::Start, "Đang nhìn màn hình…")?;
    overlay::hide_all(&app);
    tokio::time::sleep(std::time::Duration::from_millis(34)).await;
    let capture = state.capture.clone();
    let frame = tokio::task::spawn_blocking(move || capture.capture_active_display())
        .await
        .map_err(|_| internal("Không thể hoàn tất tác vụ chụp màn hình."))??;
    drop(frame);
    overlay::show_all(&app);
    state.audio.start_push_to_talk()?;
    set_assistant(&app, &state, AssistantEvent::Captured, "Đang nghe…")
}

#[tauri::command]
pub fn stop_assistant(
    app: AppHandle,
    state: State<'_, AppState>,
    reason: String,
) -> Result<(), AppError> {
    if reason.len() > 64 {
        return Err(internal("Lý do dừng không hợp lệ."));
    }
    state.cancellation().cancel();
    state.audio.stop();
    state.input.release_all()?;
    overlay::show_all(&app);
    set_assistant(&app, &state, AssistantEvent::Stop, "Sẵn sàng")
}

#[tauri::command]
pub fn start_dictation(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.audio.start_push_to_talk()?;
    set_assistant(
        &app,
        &state,
        AssistantEvent::Start,
        "Đang nghe đọc chính tả…",
    )
}

#[tauri::command]
pub fn stop_dictation(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state.audio.stop();
    set_assistant(
        &app,
        &state,
        AssistantEvent::Stop,
        "Bản xem trước đã sẵn sàng",
    )
}

fn set_assistant(
    app: &AppHandle,
    state: &State<'_, AppState>,
    event: AssistantEvent,
    status: &str,
) -> Result<(), AppError> {
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot.assistant = snapshot.assistant.transition(event)?;
        snapshot.capture_active = matches!(
            snapshot.assistant,
            AssistantState::Capturing | AssistantState::Listening
        );
        snapshot.status_vi = status.to_owned();
        snapshot.clone()
    };
    app.emit("assistant_state_changed", snapshot)
        .map_err(|_| internal("Không thể cập nhật giao diện."))
}
