use contracts::{AppError, ErrorCode};
const SERVICE: &str = "vn.tro.desktop";
const DEVICE_TOKEN: &str = "device-token";

pub fn save_device_token(token: &str) -> Result<(), AppError> {
    keyring::Entry::new(SERVICE, DEVICE_TOKEN)
        .and_then(|entry| entry.set_password(token))
        .map_err(secret_error)
}
pub fn load_device_token() -> Result<Option<zeroize::Zeroizing<String>>, AppError> {
    if let Ok(token) = std::env::var("TRO_DEVICE_TOKEN")
        && !token.trim().is_empty()
    {
        return Ok(Some(zeroize::Zeroizing::new(token)));
    }
    load(DEVICE_TOKEN).map(|value| value.map(zeroize::Zeroizing::new))
}

pub fn device_token_configured() -> bool {
    load_device_token().is_ok_and(|token| token.is_some())
}

fn load(account: &str) -> Result<Option<String>, AppError> {
    let entry = keyring::Entry::new(SERVICE, account).map_err(secret_error)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
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
