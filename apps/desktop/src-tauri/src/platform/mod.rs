#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use crate::services::audio::AudioBackend;
use contracts::{PermissionSnapshot, PermissionStatus};

pub fn permission_snapshot(audio: &dyn AudioBackend) -> PermissionSnapshot {
    PermissionSnapshot {
        microphone: if audio.microphone_available() {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Unavailable
        },
        screen_capture: PermissionStatus::NotDetermined,
        input_control: PermissionStatus::NotDetermined,
    }
}
