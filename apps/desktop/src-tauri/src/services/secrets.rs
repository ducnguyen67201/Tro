use contracts::{AppError, ErrorCode};
const SERVICE: &str = "vn.tro.desktop";
const DEVICE_TOKEN: &str = "device-token";

pub fn save_device_token(token: &str) -> Result<(), AppError> {
    keyring::Entry::new(SERVICE, DEVICE_TOKEN)
        .and_then(|entry| entry.set_password(token))
        .map_err(secret_error)
}
pub fn load_device_token() -> Result<Option<String>, AppError> {
    let entry = keyring::Entry::new(SERVICE, DEVICE_TOKEN).map_err(secret_error)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(secret_error(error)),
    }
}
fn secret_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(component = "secrets", operation = "credential_vault", error_code = "credential_vault_failed", source = %error);
    AppError::new(
        ErrorCode::Internal,
        "Không thể dùng kho thông tin bảo mật của hệ điều hành.",
        true,
    )
}
