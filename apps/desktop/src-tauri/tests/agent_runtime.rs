use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use contracts::{
    ActionLocator, ActionReceipt, ActionTarget, AgentState, AppError, ApplicationRef, CaptureScope,
    ComputerAction, ErrorCode, ForegroundContext, ImageMime, ObservationBinding,
    PlannedComputerAction, PlannerStatus, ScreenFrame, ScreenFrameMeta, UiObservationMetadata,
};
use desktop_lib::{
    domain::observation::ObservationRegistry,
    services::{
        action_executor::{ActionExecutor, ResolvedActionEvidence},
        agent_runtime::{AgentRuntime, AppApprovalDecision, RuntimeResult, RuntimeUi},
        app_approvals::AppApprovalStore,
        application::{ApplicationBackend, ApplicationIdentityState},
        computer_use::ComputerUseBackend,
        llm::LlmConfig,
        observation::{Observation, ObservationBackend, ObservationMode},
        stabilizer::Stabilizer,
        user_activity::{NativeUserActivityBackend, UserActivityBackend},
    },
};
use tokio_util::sync::CancellationToken;

struct FakeApplications;

impl ApplicationBackend for FakeApplications {
    fn catalog(&self) -> Result<Vec<ApplicationRef>, AppError> {
        Ok(vec![application()])
    }

    fn launch_or_activate(&self, _app: &ApplicationRef) -> Result<(), AppError> {
        Ok(())
    }

    fn restore_window(&self, _app_id: &str) -> Result<(), AppError> {
        Ok(())
    }

    fn identity_state(&self, app_id: &str) -> Result<ApplicationIdentityState, AppError> {
        Ok(ApplicationIdentityState {
            app_id: app_id.to_owned(),
            focused: true,
            visible: true,
        })
    }
}

struct FakeObserver {
    sequence: AtomicUsize,
}

impl ObservationBackend for FakeObserver {
    fn observe(
        &self,
        app: &ApplicationRef,
        _mode: ObservationMode,
    ) -> Result<Observation, AppError> {
        let index = self.sequence.fetch_add(1, Ordering::AcqRel);
        let binding = ObservationBinding {
            observation_id: format!("obs-{index}"),
            app_id: app.app_id.clone(),
            window_generation: 1,
            layout_generation: 1,
        };
        let metadata = UiObservationMetadata {
            binding: binding.clone(),
            capture_scope: CaptureScope::ExactWindow,
            elements: Vec::new(),
            truncated: false,
        };
        Ok(Observation::from_parts(
            metadata,
            Some(ScreenFrame {
                meta: ScreenFrameMeta {
                    frame_id: format!("frame-{index}"),
                    monitor_id: "fixture".to_owned(),
                    width_px: 1,
                    height_px: 1,
                    image_width_px: 1,
                    image_height_px: 1,
                    origin_x_px: 0,
                    origin_y_px: 0,
                    scale_factor: 1.0,
                    layout_generation: 1,
                    mime_type: ImageMime::Png,
                },
                bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
            }),
            ObservationRegistry::new(binding, []),
            ForegroundContext {
                process_hash: "fixture".to_owned(),
                window_generation: 1,
                control_role: None,
                is_secure: false,
                is_elevated: false,
            },
        ))
    }
}

struct StaleExecutor;

impl ActionExecutor for StaleExecutor {
    fn validate(
        &self,
        _app: &ApplicationRef,
        _observation: &Observation,
        _planned: &PlannedComputerAction,
    ) -> Result<ResolvedActionEvidence, AppError> {
        Err(AppError::new(ErrorCode::StaleObservation, "stale", true))
    }

    fn execute(
        &self,
        _app: &ApplicationRef,
        _observation: &Observation,
        _planned: &PlannedComputerAction,
        _cancellation: &CancellationToken,
    ) -> Result<(), AppError> {
        panic!("stale action must never execute")
    }

    fn release_all(&self) -> Result<(), AppError> {
        Ok(())
    }
}

struct FakePlanner {
    receipts: Mutex<Vec<ActionReceipt>>,
}

#[async_trait]
impl ComputerUseBackend for FakePlanner {
    async fn create_run(
        &self,
        _config: &LlmConfig,
        _goal: &str,
        _available_apps: Vec<ApplicationRef>,
        observation: &Observation,
    ) -> Result<contracts::AgentTurnResponse, AppError> {
        Ok(contracts::AgentTurnResponse {
            run_id: "run-1".to_owned(),
            turn_number: 0,
            status: PlannerStatus::Actions {
                actions: vec![PlannedComputerAction {
                    observation_id: format!(
                        "{}-stale",
                        observation.metadata.binding.observation_id
                    ),
                    locator: ActionLocator::Frame,
                    action: ComputerAction::Capture,
                    target: ActionTarget::Benign,
                    description_vi: "capture".to_owned(),
                }],
            },
        })
    }

    async fn next_turn(
        &self,
        _config: &LlmConfig,
        _goal: &str,
        _run_id: &str,
        turn_number: u32,
        receipts: Vec<ActionReceipt>,
        _observation: &Observation,
    ) -> Result<contracts::AgentTurnResponse, AppError> {
        self.receipts
            .lock()
            .expect("receipt mutex")
            .extend(receipts);
        Ok(contracts::AgentTurnResponse {
            run_id: "run-1".to_owned(),
            turn_number,
            status: PlannerStatus::Completed {
                message_vi: "Đã xong".to_owned(),
            },
        })
    }

    async fn stop_run(&self, _config: &LlmConfig, _run_id: &str) {}
}

#[derive(Default)]
struct FakeUi {
    states: Mutex<Vec<AgentState>>,
}

#[async_trait]
impl RuntimeUi for FakeUi {
    fn status(&self, state: AgentState, _message_vi: &str, _app: Option<&ApplicationRef>) {
        self.states.lock().expect("state mutex").push(state);
    }

    async fn approve_app(&self, _app: &ApplicationRef) -> Result<AppApprovalDecision, AppError> {
        Ok(AppApprovalDecision::AllowOnce)
    }

    async fn confirm_action(
        &self,
        _scope_id: uuid::Uuid,
        _app: &ApplicationRef,
        _observation: &Observation,
        _planned: &PlannedComputerAction,
    ) -> Result<bool, AppError> {
        Ok(true)
    }
}

fn application() -> ApplicationRef {
    ApplicationRef {
        app_id: "abc-browser".to_owned(),
        display_name: "ABC Browser".to_owned(),
        identity_summary: "fixture".to_owned(),
    }
}

#[tokio::test]
async fn stale_proposal_records_a_receipt_and_never_reaches_input() {
    let applications: Arc<dyn ApplicationBackend> = Arc::new(FakeApplications);
    let observer: Arc<dyn ObservationBackend> = Arc::new(FakeObserver {
        sequence: AtomicUsize::new(1),
    });
    let activity = Arc::new(NativeUserActivityBackend::manual());
    let activity_port: Arc<dyn UserActivityBackend> = activity.clone();
    let planner = Arc::new(FakePlanner {
        receipts: Mutex::new(Vec::new()),
    });
    let runtime = AgentRuntime::new(
        LlmConfig::default(),
        applications.clone(),
        Arc::new(AppApprovalStore::default()),
        observer.clone(),
        planner.clone(),
        Arc::new(StaleExecutor),
        Stabilizer::new(applications, observer, activity_port),
    );
    let ui = FakeUi::default();
    let result = runtime
        .run(
            "Open course five in ABC Browser",
            &ui,
            CancellationToken::new(),
        )
        .await
        .expect("runtime completes");
    assert_eq!(result, RuntimeResult::Completed("Đã xong".to_owned()));
    let receipts = planner.receipts.lock().expect("receipt mutex");
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].outcome, contracts::ActionOutcome::Stale);
    assert!(
        ui.states
            .lock()
            .expect("state mutex")
            .contains(&AgentState::StaleRecovery)
    );
}
