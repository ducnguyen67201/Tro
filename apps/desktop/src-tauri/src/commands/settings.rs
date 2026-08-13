use crate::{
    app_state::AppState,
    domain::settings::{AppSettings, SettingsPatch},
};
use contracts::{AppError, ErrorCode};
use tauri::State;

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    patch: SettingsPatch,
) -> Result<AppSettings, AppError> {
    let mut settings = state
        .settings
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    settings.apply(patch).map_err(|reason| {
        AppError::new(
            if reason == "shortcut conflict" {
                ErrorCode::ShortcutConflict
            } else {
                ErrorCode::InvalidRequest
            },
            "Phím tắt hoặc cài đặt không hợp lệ.",
            false,
        )
    })?;
    Ok(settings.clone())
}
