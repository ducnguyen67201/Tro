use crate::{app_state::AppState, platform};
#[cfg(target_os = "macos")]
use contracts::PermissionStatus;
use contracts::{AppError, PermissionSnapshot};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_permission_snapshot(app: AppHandle, state: State<'_, AppState>) -> PermissionSnapshot {
    permission_snapshot(&app, &state)
}

#[tauri::command]
pub fn request_permission(
    app: AppHandle,
    permission: String,
    state: State<'_, AppState>,
) -> Result<PermissionSnapshot, AppError> {
    platform::request_permission(&permission)?;
    Ok(permission_snapshot(&app, &state))
}

#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.restart();
}

#[cfg(target_os = "macos")]
fn permission_snapshot(app: &AppHandle, state: &State<'_, AppState>) -> PermissionSnapshot {
    let mut snapshot = platform::permission_snapshot(state.audio.as_ref());
    if snapshot.input_control == PermissionStatus::Granted
        && !state.command_control_shortcut.ensure_started(app)
    {
        snapshot.input_control = PermissionStatus::RestartRequired;
    }
    snapshot
}

#[cfg(not(target_os = "macos"))]
fn permission_snapshot(_app: &AppHandle, state: &State<'_, AppState>) -> PermissionSnapshot {
    platform::permission_snapshot(state.audio.as_ref())
}
