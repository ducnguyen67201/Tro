use crate::{
    app_state::AppState,
    domain::settings::{AppSettings, SettingsPatch},
};
use contracts::{AppError, ApplicationRef, ErrorCode};
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

#[tauri::command]
pub fn list_approved_apps(state: State<'_, AppState>) -> Vec<ApplicationRef> {
    state.app_approvals.list()
}

#[tauri::command]
pub fn revoke_approved_app(state: State<'_, AppState>, app_id: String) -> Result<bool, AppError> {
    if app_id.is_empty() || app_id.len() > 256 {
        return Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Định danh ứng dụng không hợp lệ.",
            false,
        ));
    }
    let removed = state.app_approvals.revoke(&app_id)?;
    if removed
        && state
            .active_app_id
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_deref()
            == Some(app_id.as_str())
    {
        state.cancellation().cancel();
    }
    Ok(removed)
}
