use contracts::{AppError, ErrorCode, PermissionSnapshot, PermissionStatus};

use crate::services::audio::AudioBackend;

pub const INPUT_LIMIT_HELP: &str =
    "Tro không chạy quyền quản trị và không thể điều khiển ứng dụng elevated do Windows UIPI.";

pub fn permission_snapshot(audio: &dyn AudioBackend) -> PermissionSnapshot {
    PermissionSnapshot {
        microphone: if audio.microphone_available() {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Unavailable
        },
        screen_capture: PermissionStatus::Granted,
        input_control: PermissionStatus::Granted,
    }
}

pub fn request_permission(permission: &str) -> Result<(), AppError> {
    match permission {
        "microphone" | "screen_capture" | "input_control" => Ok(()),
        _ => Err(AppError::new(
            ErrorCode::InvalidRequest,
            "Quyền được yêu cầu không hợp lệ.",
            false,
        )),
    }
}
