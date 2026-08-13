use contracts::{AppError, ErrorCode};
const SERVICE: &str = "vn.tro.desktop";
const DEVICE_TOKEN: &str = "device-token";
const OPENROUTER_API_KEY: &str = "openrouter-api-key";
const LLM_CONFIG: &str = "llm-config";

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

pub fn save_openrouter_api_key(api_key: &str) -> Result<(), AppError> {
    save(OPENROUTER_API_KEY, api_key)
}

pub fn load_openrouter_api_key() -> Result<Option<zeroize::Zeroizing<String>>, AppError> {
    if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY")
        && !api_key.trim().is_empty()
    {
        return Ok(Some(zeroize::Zeroizing::new(api_key)));
    }
    load(OPENROUTER_API_KEY).map(|value| value.map(zeroize::Zeroizing::new))
}

pub fn openrouter_api_key_configured() -> bool {
    load_openrouter_api_key().is_ok_and(|key| key.is_some())
}

pub fn save_llm_config(config: &str) -> Result<(), AppError> {
    save(LLM_CONFIG, config)
}

pub fn load_llm_config() -> Result<Option<String>, AppError> {
    load(LLM_CONFIG)
}

fn save(account: &str, value: &str) -> Result<(), AppError> {
    keyring::Entry::new(SERVICE, account)
        .and_then(|entry| entry.set_password(value))
        .map_err(secret_error)
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
