use contracts::{AppError, AuthSnapshot, ErrorCode};
use tauri::{AppHandle, Emitter, State};

use crate::{app_state::AppState, commands::window};

#[tauri::command]
pub async fn get_auth_snapshot(state: State<'_, AppState>) -> Result<AuthSnapshot, AppError> {
    let config = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let authenticated = state.auth.restore_session(&config).await?;
    state.set_authenticated(authenticated);
    Ok(AuthSnapshot { authenticated })
}

#[tauri::command]
pub async fn sign_in_with_invite(
    app: AppHandle,
    state: State<'_, AppState>,
    invite_code: String,
    accepted_age_scope: bool,
) -> Result<AuthSnapshot, AppError> {
    let config = state
        .llm_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    state
        .auth
        .sign_in(&config, &invite_code, accepted_age_scope)
        .await?;
    state.set_authenticated(true);
    let snapshot = AuthSnapshot {
        authenticated: true,
    };
    app.emit("authentication_changed", snapshot)
        .map_err(|_| auth_error())?;
    Ok(snapshot)
}

pub fn require_authentication(app: &AppHandle, state: &AppState) -> Result<(), AppError> {
    if state.is_authenticated() {
        return Ok(());
    }
    let _hidden = state.cursor_companion.hide(app);
    let _shown = window::show_main(app);
    let snapshot = AuthSnapshot {
        authenticated: false,
    };
    let _emitted = app.emit("authentication_changed", snapshot);
    Err(AppError::new(
        ErrorCode::AuthExpired,
        "Hãy đăng nhập vào Tro trước khi bắt đầu.",
        false,
    ))
}

pub fn handle_auth_error(app: &AppHandle, state: &AppState, error: &AppError) {
    if error.code != ErrorCode::AuthExpired {
        return;
    }
    state.set_authenticated(false);
    let _hidden = state.cursor_companion.hide(app);
    let _shown = window::show_main(app);
    let _emitted = app.emit(
        "authentication_changed",
        AuthSnapshot {
            authenticated: false,
        },
    );
}

fn auth_error() -> AppError {
    AppError::new(
        ErrorCode::Internal,
        "Tro chưa thể cập nhật trạng thái đăng nhập.",
        true,
    )
}
