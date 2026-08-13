use crate::{app_state::AppState, platform};
use contracts::{AppError, PermissionSnapshot};
use tauri::State;

#[tauri::command]
pub fn get_permission_snapshot(state: State<'_, AppState>) -> PermissionSnapshot {
    platform::permission_snapshot(state.audio.as_ref())
}

#[tauri::command]
pub fn request_permission(
    permission: String,
    state: State<'_, AppState>,
) -> Result<PermissionSnapshot, AppError> {
    platform::request_permission(&permission)?;
    Ok(platform::permission_snapshot(state.audio.as_ref()))
}
