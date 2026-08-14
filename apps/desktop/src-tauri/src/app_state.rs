use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicBool, Ordering},
};

use contracts::{AgentState, AssistantUiState, ScreenFrame};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[cfg(target_os = "macos")]
use crate::services::modifier_shortcut::CommandControlShortcut;
use crate::{
    domain::{
        confirmation::{ConfirmationChoice, ConfirmationManager},
        settings::AppSettings,
    },
    services::{
        action_executor::{ActionExecutor, SemanticFirstExecutor},
        app_approvals::AppApprovalStore,
        application::{ApplicationBackend, PlatformApplicationBackend},
        audio::{AudioBackend, CpalAudioBackend},
        auth::AuthGateway,
        capture::{CaptureBackend, XcapCaptureBackend},
        computer_use::{ComputerUseBackend, ComputerUseGateway},
        cursor_companion::CursorCompanion,
        input::{InputBackend, NativeInputBackend},
        llm::{LlmConfig, LlmGateway},
        observation::{ObservationBackend, PlatformObservationBackend},
        speech::{NativeSpeechBackend, SpeechBackend},
        user_activity::{NativeUserActivityBackend, UserActivityBackend},
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
    pub computer_use: Arc<dyn ComputerUseBackend>,
    pub llm_config: RwLock<LlmConfig>,
    pub input: Arc<dyn InputBackend>,
    pub speech: Arc<dyn SpeechBackend>,
    pub confirmation: Mutex<ConfirmationManager>,
    pub app_approvals: Arc<AppApprovalStore>,
    pub applications: Arc<dyn ApplicationBackend>,
    pub observer: Arc<dyn ObservationBackend>,
    pub action_executor: Arc<dyn ActionExecutor>,
    pub user_activity: Arc<NativeUserActivityBackend>,
    pub active_app_id: RwLock<Option<String>>,
    confirmation_waiter: Mutex<Option<ConfirmationWaiter>>,
    pub cursor_companion: CursorCompanion,
    authenticated: AtomicBool,
    #[cfg(target_os = "macos")]
    pub command_control_shortcut: CommandControlShortcut,
    cancellation: RwLock<CancellationToken>,
}

impl AppState {
    pub fn new() -> Self {
        let capture: Arc<dyn CaptureBackend> = Arc::new(XcapCaptureBackend);
        let input: Arc<dyn InputBackend> = Arc::new(NativeInputBackend);
        let applications: Arc<dyn ApplicationBackend> = Arc::new(PlatformApplicationBackend);
        let observer: Arc<dyn ObservationBackend> =
            Arc::new(PlatformObservationBackend::new(capture.clone()));
        let user_activity = Arc::new(NativeUserActivityBackend::default());
        let activity_port: Arc<dyn UserActivityBackend> = user_activity.clone();
        let action_executor: Arc<dyn ActionExecutor> = Arc::new(SemanticFirstExecutor::new(
            applications.clone(),
            observer.clone(),
            input.clone(),
            activity_port,
        ));
        Self {
            snapshot: RwLock::new(AssistantUiState::default()),
            settings: RwLock::new(AppSettings::default()),
            capture,
            audio: Arc::new(CpalAudioBackend::default()),
            auth: AuthGateway::default(),
            pending_frame: Mutex::new(None),
            frame_ready: Notify::new(),
            llm: LlmGateway::default(),
            computer_use: Arc::new(ComputerUseGateway::default()),
            llm_config: RwLock::new(LlmConfig::load()),
            input,
            speech: Arc::new(NativeSpeechBackend::default()),
            confirmation: Mutex::new(ConfirmationManager::default()),
            app_approvals: Arc::new(AppApprovalStore::default()),
            applications,
            observer,
            action_executor,
            user_activity,
            active_app_id: RwLock::new(None),
            confirmation_waiter: Mutex::new(None),
            cursor_companion: CursorCompanion::default(),
            // A stored token is not trusted until the backend refreshes it.
            authenticated: AtomicBool::new(false),
            #[cfg(target_os = "macos")]
            command_control_shortcut: CommandControlShortcut::default(),
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
        self.confirmation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.cancel_confirmation_waiter();
        *self
            .active_app_id
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        let _release = self.input.release_all();
        let mut snapshot = self
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *snapshot = AssistantUiState::default();
        snapshot.agent = AgentState::Idle;
    }

    pub fn wait_for_confirmation(
        &self,
        id: Uuid,
    ) -> tokio::sync::oneshot::Receiver<ConfirmationChoice> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let previous = self
            .confirmation_waiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(ConfirmationWaiter { id, sender });
        if let Some(previous) = previous {
            let _result = previous.sender.send(ConfirmationChoice::Stop);
        }
        receiver
    }

    pub fn resolve_confirmation_waiter(&self, id: Uuid, decision: ConfirmationChoice) -> bool {
        let waiter = self
            .confirmation_waiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let Some(waiter) = waiter else {
            return false;
        };
        if waiter.id != id {
            let _result = waiter.sender.send(ConfirmationChoice::Stop);
            return false;
        }
        waiter.sender.send(decision).is_ok()
    }

    pub fn cancel_confirmation_waiter(&self) {
        if let Some(waiter) = self
            .confirmation_waiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _result = waiter.sender.send(ConfirmationChoice::Stop);
        }
    }
}

struct ConfirmationWaiter {
    id: Uuid,
    sender: tokio::sync::oneshot::Sender<ConfirmationChoice>,
}
