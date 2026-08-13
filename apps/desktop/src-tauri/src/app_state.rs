use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicBool, Ordering},
};

use contracts::{AgentState, AssistantUiState, ScreenFrame};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[cfg(target_os = "macos")]
use crate::services::modifier_shortcut::CommandOptionShortcut;
use crate::{
    domain::{confirmation::ConfirmationManager, settings::AppSettings},
    services::{
        audio::{AudioBackend, CpalAudioBackend},
        auth::AuthGateway,
        capture::{CaptureBackend, XcapCaptureBackend},
        computer_use::ComputerUseGateway,
        cursor_companion::CursorCompanion,
        foreground::{ForegroundContextBackend, PlatformForegroundBackend},
        input::{InputBackend, NativeInputBackend},
        llm::{LlmConfig, LlmGateway},
        speech::{NativeSpeechBackend, SpeechBackend},
    },
};

pub struct AppState {
    pub snapshot: RwLock<AssistantUiState>,
    pub settings: RwLock<AppSettings>,
    pub capture: Arc<dyn CaptureBackend>,
    pub audio: Arc<dyn AudioBackend>,
    pub auth: AuthGateway,
    pub pending_frame: Mutex<Option<ScreenFrame>>,
    pub frame_ready: Notify,
    pub llm: LlmGateway,
    pub computer_use: ComputerUseGateway,
    pub llm_config: RwLock<LlmConfig>,
    pub input: Arc<dyn InputBackend>,
    pub foreground: Arc<dyn ForegroundContextBackend>,
    pub speech: Arc<dyn SpeechBackend>,
    pub confirmation: Mutex<ConfirmationManager>,
    confirmation_waiter: Mutex<Option<ConfirmationWaiter>>,
    pub cursor_companion: CursorCompanion,
    authenticated: AtomicBool,
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
            audio: Arc::new(CpalAudioBackend::default()),
            auth: AuthGateway::default(),
            pending_frame: Mutex::new(None),
            frame_ready: Notify::new(),
            llm: LlmGateway::default(),
            computer_use: ComputerUseGateway::default(),
            llm_config: RwLock::new(LlmConfig::load()),
            input: Arc::new(NativeInputBackend),
            foreground: Arc::new(PlatformForegroundBackend),
            speech: Arc::new(NativeSpeechBackend::default()),
            confirmation: Mutex::new(ConfirmationManager::default()),
            confirmation_waiter: Mutex::new(None),
            cursor_companion: CursorCompanion::default(),
            // A stored token is not trusted until the backend refreshes it.
            authenticated: AtomicBool::new(false),
            #[cfg(target_os = "macos")]
            command_option_shortcut: CommandOptionShortcut::default(),
            cancellation: RwLock::new(CancellationToken::new()),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated.load(Ordering::Acquire)
    }

    pub fn set_authenticated(&self, authenticated: bool) {
        self.authenticated.store(authenticated, Ordering::Release);
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
        self.audio.stop();
        self.speech.stop();
        self.pending_frame
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let _release = self.input.release_all();
        let mut snapshot = self
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *snapshot = AssistantUiState::default();
        snapshot.agent = AgentState::Idle;
    }

    pub fn wait_for_confirmation(&self, id: Uuid) -> tokio::sync::oneshot::Receiver<bool> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let previous = self
            .confirmation_waiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(ConfirmationWaiter { id, sender });
        if let Some(previous) = previous {
            let _result = previous.sender.send(false);
        }
        receiver
    }

    pub fn resolve_confirmation_waiter(&self, id: Uuid, allowed: bool) -> bool {
        let waiter = self
            .confirmation_waiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let Some(waiter) = waiter else {
            return false;
        };
        if waiter.id != id {
            let _result = waiter.sender.send(false);
            return false;
        }
        waiter.sender.send(allowed).is_ok()
    }

    pub fn cancel_confirmation_waiter(&self) {
        if let Some(waiter) = self
            .confirmation_waiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _result = waiter.sender.send(false);
        }
    }
}

struct ConfirmationWaiter {
    id: Uuid,
    sender: tokio::sync::oneshot::Sender<bool>,
}
