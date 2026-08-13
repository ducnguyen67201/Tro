use contracts::{AppError, CursorCompanionSnapshot};
use tauri::{AppHandle, Manager, State};

use crate::{app_state::AppState, domain::error::internal};

#[tauri::command]
pub fn get_cursor_companion_snapshot(state: State<'_, AppState>) -> CursorCompanionSnapshot {
    state.cursor_companion.snapshot()
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), AppError> {
    show_main(&app)
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), AppError> {
    hide_main(&app)
}

#[tauri::command]
pub fn follow_cursor_companion(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    state
        .cursor_companion
        .follow(&app)
        .map_err(|error| window_error("follow_cursor_companion", error))
}

#[tauri::command]
pub fn dismiss_cursor_companion(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state
        .cursor_companion
        .hide(&app)
        .map_err(|error| window_error("dismiss_cursor_companion", error))
}

pub fn show_main(app: &AppHandle) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Regular)
            .map_err(|error| window_error("set_regular_activation", error))?;
        app.set_dock_visibility(true)
            .map_err(|error| window_error("show_dock", error))?;
    }

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| internal("Không tìm thấy cửa sổ Tro."))?;
    window
        .unminimize()
        .map_err(|error| window_error("unminimize_main", error))?;
    window
        .show()
        .map_err(|error| window_error("show_main", error))?;
    window
        .set_focus()
        .map_err(|error| window_error("focus_main", error))
}

pub fn hide_main(app: &AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|error| window_error("hide_main", error))?;
    }

    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory)
            .map_err(|error| window_error("set_accessory_activation", error))?;
        app.set_dock_visibility(false)
            .map_err(|error| window_error("hide_dock", error))?;
    }
    Ok(())
}

fn window_error(operation: &'static str, error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "window",
        operation,
        error_code = "window_operation_failed",
        source = %error
    );
    internal("Tro chưa thể cập nhật cửa sổ.")
}
