use std::sync::{Arc, Mutex, RwLock};

use contracts::{AgentState, AssistantUiState};
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
use crate::services::modifier_shortcut::CommandOptionShortcut;
use crate::{
    domain::{confirmation::ConfirmationManager, settings::AppSettings},
    services::{
        audio::{AudioBackend, CpalAudioBackend},
        capture::{CaptureBackend, XcapCaptureBackend},
        foreground::{ForegroundContextBackend, PlatformForegroundBackend},
        input::{InputBackend, NativeInputBackend},
    },
};

pub struct AppState {
    pub snapshot: RwLock<AssistantUiState>,
    pub settings: RwLock<AppSettings>,
    pub capture: Arc<dyn CaptureBackend>,
    pub audio: Arc<dyn AudioBackend>,
    pub input: Arc<dyn InputBackend>,
    pub foreground: Arc<dyn ForegroundContextBackend>,
    pub confirmation: Mutex<ConfirmationManager>,
    #[cfg(target_os = "macos")]
    pub command_option_shortcut: CommandOptionShortcut,
    cancellation: RwLock<CancellationToken>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            snapshot: RwLock::new(AssistantUiState::default()),
            settings: RwLock::new(AppSettings::default()),
            capture: Arc::new(XcapCaptureBackend),
            audio: Arc::new(CpalAudioBackend),
            input: Arc::new(NativeInputBackend),
            foreground: Arc::new(PlatformForegroundBackend),
            confirmation: Mutex::new(ConfirmationManager::default()),
            #[cfg(target_os = "macos")]
            command_option_shortcut: CommandOptionShortcut::default(),
            cancellation: RwLock::new(CancellationToken::new()),
        }
    }

    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn reset_cancellation(&self) -> CancellationToken {
        let mut guard = self
            .cancellation
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.cancel();
        *guard = CancellationToken::new();
        guard.clone()
    }

    pub fn reset_after_restart(&self) {
        self.cancellation().cancel();
        let _release = self.input.release_all();
        let mut snapshot = self
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *snapshot = AssistantUiState::default();
        snapshot.agent = AgentState::Idle;
    }
}
