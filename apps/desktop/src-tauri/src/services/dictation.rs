use contracts::{AppError, ErrorCode, ForegroundContext, SecretText};
use unicode_normalization::UnicodeNormalization;

pub fn prepare_insertion(
    text: &str,
    observed_generation: u64,
    current: &ForegroundContext,
) -> Result<SecretText, AppError> {
    if text.is_empty() || text.len() > 10_000 {
        return Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Văn bản đọc chính tả không hợp lệ.",
            false,
        ));
    }
    if current.is_secure || current.control_role.as_deref() == Some("password") {
        return Err(AppError::new(
            ErrorCode::ActionBlocked,
            "Không thể chèn văn bản vào trường bảo mật.",
            false,
        ));
    }
    if current.window_generation != observed_generation {
        return Err(AppError::new(
            ErrorCode::ActionRequiresConfirmation,
            "Cửa sổ đã thay đổi. Hãy xem lại trước khi chèn.",
            false,
        ));
    }
    Ok(SecretText::new(text.nfc().collect::<String>()))
}
