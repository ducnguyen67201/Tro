use crate::{
    app_state::AppState,
    domain::settings::{AppSettings, SettingsPatch},
};
use contracts::{AppError, ErrorCode};
use tauri::State;

use crate::services::llm::LlmConfigSnapshot;

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

#[tauri::command]
pub fn get_llm_config(state: State<'_, AppState>) -> LlmConfigSnapshot {
    let config = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    LlmConfigSnapshot::from_config(&config)
}
