use contracts::{AppError, ErrorCode, PermissionSnapshot, PermissionStatus};
use ghost_permissions::{Capability, granted, request};

use crate::services::audio::AudioBackend;

pub fn permission_snapshot(audio: &dyn AudioBackend) -> PermissionSnapshot {
    PermissionSnapshot {
        microphone: microphone_status(audio),
        screen_capture: if granted(Capability::ScreenRecording) {
            PermissionStatus::Granted
        } else {
            PermissionStatus::NotDetermined
        },
        input_control: if granted(Capability::Accessibility) && granted(Capability::InputMonitoring)
        {
            PermissionStatus::Granted
        } else {
            PermissionStatus::NotDetermined
        },
    }
}

pub fn request_permission(permission: &str) -> Result<(), AppError> {
    match permission {
        "microphone" => Ok(()),
        "screen_capture" => {
            request(Capability::ScreenRecording);
            Ok(())
        }
        "input_control" => {
            request(Capability::InputMonitoring);
            request(Capability::Accessibility);
            Ok(())
        }
        _ => Err(invalid_permission()),
    }
}

fn microphone_status(audio: &dyn AudioBackend) -> PermissionStatus {
    if audio.microphone_available() {
        PermissionStatus::Granted
    } else {
        PermissionStatus::Unavailable
    }
}

fn invalid_permission() -> AppError {
    AppError::new(
        ErrorCode::InvalidRequest,
        "Quyền được yêu cầu không hợp lệ.",
        false,
    )
}

#[cfg(test)]
mod tests {
    use contracts::PermissionStatus;

    use super::{permission_snapshot, request_permission};
    use crate::services::audio::AudioBackend;

    struct AvailableMicrophone;

    impl AudioBackend for AvailableMicrophone {
        fn microphone_available(&self) -> bool {
            true
        }

        fn start_push_to_talk(&self) -> Result<(), contracts::AppError> {
            Ok(())
        }

        fn finish_push_to_talk(
            &self,
        ) -> Result<crate::services::audio::RecordedAudio, contracts::AppError> {
            Ok(crate::services::audio::RecordedAudio {
                wav_bytes: vec![0; 44],
            })
        }

        fn stop(&self) {}
    }

    #[test]
    fn rejects_unknown_permission_without_prompting() {
        let error = request_permission("camera").expect_err("unknown permission must fail");
        assert_eq!(error.code.as_str(), "invalid_request");
    }

    #[test]
    fn reads_permission_status_without_opening_prompts() {
        let snapshot = permission_snapshot(&AvailableMicrophone);
        assert_eq!(snapshot.microphone, PermissionStatus::Granted);
        assert!(matches!(
            snapshot.screen_capture,
            PermissionStatus::Granted | PermissionStatus::NotDetermined
        ));
        assert!(matches!(
            snapshot.input_control,
            PermissionStatus::Granted | PermissionStatus::NotDetermined
        ));
    }
}
