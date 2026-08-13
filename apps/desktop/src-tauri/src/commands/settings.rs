use crate::{
    app_state::AppState,
    domain::settings::{AppSettings, SettingsPatch},
};
use contracts::{AppError, ErrorCode};
use tauri::State;
use zeroize::Zeroizing;

use crate::services::{
    llm::{LlmConfigPatch, LlmConfigSnapshot},
    secrets,
};

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

#[tauri::command]
pub fn update_llm_config(
    state: State<'_, AppState>,
    patch: LlmConfigPatch,
) -> Result<LlmConfigSnapshot, AppError> {
    let current = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let (config, api_key) = patch.apply(&current)?;
    if let Some(api_key) = api_key {
        let api_key = Zeroizing::new(api_key);
        secrets::save_openrouter_api_key(api_key.as_str())?;
    }
    let serialized = serde_json::to_string(&config)
        .map_err(|_| AppError::new(ErrorCode::Internal, "Không thể lưu cấu hình LLM.", true))?;
    secrets::save_llm_config(&serialized)?;
    *state
        .llm_config
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = config.clone();
    Ok(LlmConfigSnapshot::from_config(&config))
}
