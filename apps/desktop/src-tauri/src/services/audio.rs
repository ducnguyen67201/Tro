use contracts::{AppError, ErrorCode};
use cpal::traits::HostTrait;

pub trait AudioBackend: Send + Sync {
    fn microphone_available(&self) -> bool;
    fn start_push_to_talk(&self) -> Result<(), AppError>;
    fn stop(&self);
}

pub struct CpalAudioBackend;

impl AudioBackend for CpalAudioBackend {
    fn microphone_available(&self) -> bool {
        cpal::default_host().default_input_device().is_some()
    }
    fn start_push_to_talk(&self) -> Result<(), AppError> {
        if self.microphone_available() {
            Ok(())
        } else {
            Err(AppError::new(
                ErrorCode::MicrophoneUnavailable,
                "Không tìm thấy micrô. Bạn vẫn có thể nhập câu hỏi.",
                true,
            ))
        }
    }
    fn stop(&self) {}
}
