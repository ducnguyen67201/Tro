use crate::{app_state::AppState, platform};
use contracts::{AppError, ErrorCode, PermissionSnapshot};
use tauri::State;

#[tauri::command]
pub fn get_permission_snapshot(state: State<'_, AppState>) -> PermissionSnapshot {
    platform::permission_snapshot(state.audio.as_ref())
}

#[tauri::command]
pub fn request_permission(permission: String) -> Result<(), AppError> {
    match permission.as_str() {
        "microphone" | "screen_capture" | "input_control" => Ok(()),
        _ => Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Quyền được yêu cầu không hợp lệ.",
            false,
        )),
    }
}
