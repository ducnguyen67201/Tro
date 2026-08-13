use contracts::{AppError, ErrorCode};

pub fn internal(message_vi: &'static str) -> AppError {
    AppError::new(ErrorCode::Internal, message_vi, true)
}
