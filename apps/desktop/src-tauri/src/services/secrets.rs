use std::sync::{Mutex, OnceLock};

use contracts::AppError;
use zeroize::Zeroizing;

static DEVICE_TOKEN: OnceLock<Mutex<Option<Zeroizing<String>>>> = OnceLock::new();
static DEVICE_PUBLIC_ID: OnceLock<String> = OnceLock::new();

fn device_token_slot() -> &'static Mutex<Option<Zeroizing<String>>> {
    DEVICE_TOKEN.get_or_init(|| Mutex::new(None))
}

pub fn save_device_token(token: &str) -> Result<(), AppError> {
    let mut slot = device_token_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *slot = Some(Zeroizing::new(token.to_owned()));
    Ok(())
}

pub fn load_device_token() -> Result<Option<Zeroizing<String>>, AppError> {
    if let Ok(token) = std::env::var("TRO_DEVICE_TOKEN")
        && !token.trim().is_empty()
    {
        return Ok(Some(Zeroizing::new(token)));
    }
    let slot = device_token_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(slot.clone())
}

pub fn device_token_configured() -> bool {
    load_device_token().is_ok_and(|token| token.is_some())
}

pub fn delete_device_token() -> Result<(), AppError> {
    if std::env::var("TRO_DEVICE_TOKEN").is_ok() {
        return Ok(());
    }
    let mut slot = device_token_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *slot = None;
    Ok(())
}

pub fn load_or_create_device_public_id() -> Result<String, AppError> {
    Ok(DEVICE_PUBLIC_ID
        .get_or_init(|| uuid::Uuid::new_v4().to_string())
        .clone())
}

#[cfg(test)]
mod tests {
    use super::{delete_device_token, load_device_token, save_device_token};

    #[test]
    fn device_session_is_kept_only_in_process_memory() {
        delete_device_token().unwrap();
        assert!(load_device_token().unwrap().is_none());

        save_device_token("opaque-device-session").unwrap();
        assert_eq!(
            load_device_token()
                .unwrap()
                .as_ref()
                .map(|token| token.as_str()),
            Some("opaque-device-session")
        );

        delete_device_token().unwrap();
        assert!(load_device_token().unwrap().is_none());
    }
}
