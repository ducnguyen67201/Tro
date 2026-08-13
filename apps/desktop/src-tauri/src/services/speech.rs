use std::{
    io::Write,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use contracts::AppError;

use crate::domain::error::internal;

const MAX_SPEECH_CHARACTERS: usize = 1_200;

pub trait SpeechBackend: Send + Sync {
    fn speak(&self, text: &str) -> Result<(), AppError>;
    fn stop(&self);
}

#[derive(Default)]
pub struct NativeSpeechBackend {
    active: Mutex<Option<Arc<Mutex<Child>>>>,
}

impl SpeechBackend for NativeSpeechBackend {
    fn speak(&self, text: &str) -> Result<(), AppError> {
        let text = validate_text(text)?;
        self.stop();

        let mut command = speech_command()?;
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| internal("Tro chưa thể bật giọng đọc trên máy."))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| internal("Tro chưa thể gửi nội dung đến giọng đọc."))?;
        stdin
            .write_all(text.as_bytes())
            .map_err(|_| internal("Tro chưa thể gửi nội dung đến giọng đọc."))?;
        drop(stdin);

        let child = Arc::new(Mutex::new(child));
        *self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(child.clone());

        let status = loop {
            let status = child
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .try_wait()
                .map_err(|_| internal("Giọng đọc của Tro đã dừng ngoài dự kiến."))?;
            if let Some(status) = status {
                break status;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &child))
        {
            active.take();
        }
        if status.success() {
            Ok(())
        } else {
            Err(internal("Tro chưa thể hoàn tất giọng đọc trên máy."))
        }
    }

    fn stop(&self) {
        let child = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(child) = child {
            let mut child = child
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let _killed = child.kill();
            let _waited = child.wait();
        }
    }
}

pub async fn speak_best_effort(backend: Arc<dyn SpeechBackend>, text: String) {
    let result = tokio::task::spawn_blocking(move || backend.speak(&text)).await;
    if !matches!(result, Ok(Ok(()))) {
        tracing::warn!(
            component = "speech",
            operation = "speak_response",
            error_code = "speech_unavailable"
        );
    }
}

fn validate_text(text: &str) -> Result<&str, AppError> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > MAX_SPEECH_CHARACTERS || text.contains('\0') {
        return Err(internal("Nội dung giọng đọc không hợp lệ."));
    }
    Ok(text)
}

#[cfg(target_os = "macos")]
fn speech_command() -> Result<Command, AppError> {
    let mut command = Command::new("/usr/bin/say");
    command.args(["-v", "Linh"]);
    Ok(command)
}

#[cfg(target_os = "windows")]
fn speech_command() -> Result<Command, AppError> {
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$text=[Console]::In.ReadToEnd(); Add-Type -AssemblyName System.Speech; $voice=New-Object System.Speech.Synthesis.SpeechSynthesizer; try { $voice.SelectVoiceByHints([System.Speech.Synthesis.VoiceGender]::NotSet,[System.Speech.Synthesis.VoiceAge]::NotSet,0,[Globalization.CultureInfo]::GetCultureInfo('vi-VN')) } catch {}; $voice.Speak($text)",
    ]);
    Ok(command)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn speech_command() -> Result<Command, AppError> {
    Err(internal("Giọng đọc hiện chỉ hỗ trợ macOS và Windows."))
}

#[cfg(test)]
mod tests {
    use super::validate_text;

    #[test]
    fn rejects_empty_or_nul_speech() {
        assert!(validate_text("   ").is_err());
        assert!(validate_text("xin chào\0").is_err());
    }

    #[test]
    fn accepts_trimmed_vietnamese_speech() {
        assert_eq!(
            validate_text("  Mình đã mở Chrome.  ").unwrap(),
            "Mình đã mở Chrome."
        );
    }
}
