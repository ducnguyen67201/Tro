use crate::{
    app_state::AppState,
    domain::error::internal,
    services::{llm::LlmTurnInput, overlay},
};
use contracts::{AppError, AssistantEvent, AssistantState, AssistantUiState, PhysicalPoint};
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
    let token = state.reset_cancellation();
    state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    set_assistant(&app, &state, AssistantEvent::Start, "Đang nhìn màn hình…")?;
    if let Err(error) = state.audio.start_push_to_talk() {
        return fail_and_show(&app, &state, error);
    }
    overlay::hide_all(&app);
    tokio::time::sleep(std::time::Duration::from_millis(34)).await;
    let cursor = app.cursor_position().ok().map(|position| PhysicalPoint {
        x: position.x.round() as i32,
        y: position.y.round() as i32,
    });
    let capture = state.capture.clone();
    let capture_result =
        tokio::task::spawn_blocking(move || capture.capture_display_at(cursor)).await;
    overlay::show_all(&app);
    let frame = match capture_result {
        Ok(Ok(frame)) => frame,
        Ok(Err(error)) => {
            return fail_and_show(&app, &state, error);
        }
        Err(_) => {
            let error = internal("Không thể hoàn tất tác vụ chụp màn hình.");
            return fail_and_show(&app, &state, error);
        }
    };
    if token.is_cancelled() {
        return Ok(());
    }
    *state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(frame);
    let _transitioned = set_assistant_if_current(
        &app,
        &state,
        AssistantState::Capturing,
        AssistantEvent::Captured,
        "Đang nghe…",
    )?;
    Ok(())
}

#[tauri::command]
pub async fn finish_assistant(
    app: AppHandle,
    state: State<'_, AppState>,
    reason: String,
) -> Result<(), AppError> {
    if reason.len() > 64 {
        return Err(internal("Lý do kết thúc không hợp lệ."));
    }
    overlay::show_all(&app);
    let current = state
        .snapshot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .assistant;
    if !matches!(
        current,
        AssistantState::Capturing | AssistantState::Listening
    ) {
        return Ok(());
    }
    let mut audio = match state.audio.finish_push_to_talk() {
        Ok(audio) => audio,
        Err(error) => return fail_and_show(&app, &state, error),
    };
    let mut frame = match state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
    {
        Some(frame) => frame,
        None => {
            return fail_and_show(
                &app,
                &state,
                AppError::new(
                    contracts::ErrorCode::CaptureFailed,
                    "Tro chưa có ảnh màn hình cho câu hỏi này. Hãy thử lại.",
                    true,
                ),
            );
        }
    };
    set_assistant(&app, &state, AssistantEvent::Heard, "Đang gửi đến LLM…")?;
    let config = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let token = state.cancellation();
    let input = LlmTurnInput {
        audio_wav: std::mem::take(&mut audio.wav_bytes),
        screenshot_jpeg: std::mem::take(&mut frame.bytes),
    };
    let result = tokio::select! {
        () = token.cancelled() => return Ok(()),
        result = state.llm.complete(&config, input) => result,
    };
    match result {
        Ok(response) => {
            set_assistant_response(&app, &state, response)?;
            show_result_card(&app, &state);
            Ok(())
        }
        Err(error) => fail_and_show(&app, &state, error),
    }
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
    state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    state.input.release_all()?;
    overlay::show_all(&app);
    if let Err(error) = state.cursor_companion.follow(&app) {
        tracing::warn!(
            component = "cursor_companion",
            operation = "restore_after_stop",
            error_code = "window_operation_failed",
            source = %error
        );
    }
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
        if event == AssistantEvent::Start {
            snapshot.transcript = None;
        }
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

fn set_assistant_response(
    app: &AppHandle,
    state: &State<'_, AppState>,
    response: String,
) -> Result<(), AppError> {
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot.assistant = snapshot.assistant.transition(AssistantEvent::Guidance)?;
        snapshot.transcript = Some(response);
        snapshot.capture_active = false;
        snapshot.status_vi = "Tro đã trả lời".to_owned();
        snapshot.clone()
    };
    app.emit("assistant_state_changed", snapshot)
        .map_err(|_| internal("Không thể cập nhật giao diện."))
}

fn fail_and_show(
    app: &AppHandle,
    state: &State<'_, AppState>,
    error: AppError,
) -> Result<(), AppError> {
    state.audio.stop();
    state
        .pending_frame
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let _failed = set_assistant(app, state, AssistantEvent::Fail, &error.message_vi);
    show_result_card(app, state);
    Err(error)
}

fn show_result_card(app: &AppHandle, state: &State<'_, AppState>) {
    if let Err(error) = state.cursor_companion.anchor(app) {
        tracing::warn!(
            component = "cursor_companion",
            operation = "show_assistant_result",
            error_code = "window_operation_failed",
            source = %error
        );
    }
}

fn set_assistant_if_current(
    app: &AppHandle,
    state: &State<'_, AppState>,
    expected: AssistantState,
    event: AssistantEvent,
    status: &str,
) -> Result<bool, AppError> {
    let snapshot = {
        let mut snapshot = state
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if snapshot.assistant != expected {
            return Ok(false);
        }
        snapshot.assistant = snapshot.assistant.transition(event)?;
        snapshot.capture_active = matches!(
            snapshot.assistant,
            AssistantState::Capturing | AssistantState::Listening
        );
        snapshot.status_vi = status.to_owned();
        snapshot.clone()
    };
    app.emit("assistant_state_changed", snapshot)
        .map_err(|_| internal("Không thể cập nhật giao diện."))?;
    Ok(true)
}
