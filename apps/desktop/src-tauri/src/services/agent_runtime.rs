use std::sync::Arc;

use async_trait::async_trait;
use contracts::{
    ActionOutcome, ActionReceipt, ActionReceiptEvidence, AgentState, AppError, ApplicationRef,
    ComputerAction, ErrorCode, PlannerStatus, PolicyReason, RiskTier,
};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::app_scope::RunScope,
    security::action_policy::{ActionContext, ActionPolicy},
    services::{
        action_executor::{ActionExecutor, ResolvedActionEvidence},
        app_approvals::AppApprovalStore,
        application::{ApplicationBackend, ApplicationResolution},
        computer_use::ComputerUseBackend,
        llm::LlmConfig,
        observation::{Observation, ObservationBackend, ObservationMode},
        stabilizer::Stabilizer,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppApprovalDecision {
    AllowOnce,
    AlwaysAllow,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeResult {
    Completed(String),
    NeedsUser {
        reason_code: String,
        message_vi: String,
        choices: Vec<ApplicationRef>,
    },
    PausedByUser,
}

#[async_trait]
pub trait RuntimeUi: Send + Sync {
    fn status(&self, state: AgentState, message_vi: &str, app: Option<&ApplicationRef>);

    async fn approve_app(&self, app: &ApplicationRef) -> Result<AppApprovalDecision, AppError>;

    async fn confirm_action(
        &self,
        scope_id: uuid::Uuid,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &contracts::PlannedComputerAction,
    ) -> Result<bool, AppError>;
}

pub struct AgentRuntime {
    config: LlmConfig,
    applications: Arc<dyn ApplicationBackend>,
    approvals: Arc<AppApprovalStore>,
    observer: Arc<dyn ObservationBackend>,
    planner: Arc<dyn ComputerUseBackend>,
    executor: Arc<dyn ActionExecutor>,
    stabilizer: Stabilizer,
}

impl AgentRuntime {
    pub fn new(
        config: LlmConfig,
        applications: Arc<dyn ApplicationBackend>,
        approvals: Arc<AppApprovalStore>,
        observer: Arc<dyn ObservationBackend>,
        planner: Arc<dyn ComputerUseBackend>,
        executor: Arc<dyn ActionExecutor>,
        stabilizer: Stabilizer,
    ) -> Self {
        Self {
            config,
            applications,
            approvals,
            observer,
            planner,
            executor,
            stabilizer,
        }
    }

    pub async fn run(
        &self,
        goal: &str,
        ui: &dyn RuntimeUi,
        cancellation: CancellationToken,
    ) -> Result<RuntimeResult, AppError> {
        self.run_for_app(goal, None, ui, cancellation).await
    }

    pub async fn run_for_app(
        &self,
        goal: &str,
        requested_app_id: Option<&str>,
        ui: &dyn RuntimeUi,
        cancellation: CancellationToken,
    ) -> Result<RuntimeResult, AppError> {
        ui.status(AgentState::ResolvingApp, "Đang tìm ứng dụng phù hợp…", None);
        let catalog = self.applications.catalog()?;
        let resolution = requested_app_id.map_or_else(
            || self.applications.resolve(goal),
            |app_id| {
                Ok(catalog
                    .iter()
                    .find(|app| app.app_id == app_id)
                    .cloned()
                    .map_or(
                        ApplicationResolution::NotFound,
                        ApplicationResolution::Match,
                    ))
            },
        )?;
        let app = match resolution {
            ApplicationResolution::Match(app) => app,
            ApplicationResolution::Ambiguous(choices) => {
                ui.status(
                    AgentState::NeedsUser,
                    "Có nhiều ứng dụng phù hợp; hãy chọn ứng dụng bạn muốn dùng.",
                    None,
                );
                return Ok(RuntimeResult::NeedsUser {
                    reason_code: ErrorCode::AmbiguousApp.as_str().to_owned(),
                    message_vi: "Có nhiều ứng dụng phù hợp; hãy chọn ứng dụng bạn muốn dùng."
                        .to_owned(),
                    choices,
                });
            }
            ApplicationResolution::NotFound => {
                return Ok(RuntimeResult::NeedsUser {
                    reason_code: ErrorCode::TargetAppUnavailable.as_str().to_owned(),
                    message_vi:
                        "Tro chưa xác định được ứng dụng trong yêu cầu. Hãy nói rõ tên ứng dụng."
                            .to_owned(),
                    choices: catalog.into_iter().take(3).collect(),
                });
            }
        };

        let mut scope = RunScope::new(goal);
        if self.approvals.is_always_allowed(&app.app_id) {
            scope.approve_app(app.app_id.clone());
        } else {
            ui.status(
                AgentState::AwaitingAppApproval,
                "Tro đang chờ bạn cho phép ứng dụng này.",
                Some(&app),
            );
            match ui.approve_app(&app).await? {
                AppApprovalDecision::AllowOnce => scope.approve_app(app.app_id.clone()),
                AppApprovalDecision::AlwaysAllow => {
                    self.approvals.allow_always(app.clone())?;
                    scope.approve_app(app.app_id.clone());
                }
                AppApprovalDecision::Stop => return Err(cancelled()),
            }
        }

        ui.status(
            AgentState::ActivatingApp,
            "Đang mở và kiểm tra đúng ứng dụng…",
            Some(&app),
        );
        self.applications.launch_or_activate(&app)?;
        let mut observation = match self
            .stabilizer
            .wait_for_activation(&app, &cancellation)
            .await
        {
            Ok(observation) => observation,
            Err(error) if error.code == ErrorCode::UserTakeover => {
                let _release = self.executor.release_all();
                return Ok(RuntimeResult::PausedByUser);
            }
            Err(error) => return Err(error),
        };
        scope.bind(observation.metadata.binding.clone())?;
        ui.status(
            AgentState::Planning,
            "Đang lập kế hoạch từ giao diện mới nhất…",
            Some(&app),
        );
        let planning_activity = self.stabilizer.activity_snapshot();
        let mut response = tokio::select! {
            result = self.planner.create_run(&self.config, goal, catalog, &observation) => result?,
            error = self.stabilizer.wait_for_takeover(planning_activity, &cancellation) => {
                let _release = self.executor.release_all();
                return if error.code == ErrorCode::UserTakeover {
                    Ok(RuntimeResult::PausedByUser)
                } else {
                    Err(error)
                };
            }
        };
        let mut run_id = Some(response.run_id.clone());
        if let Err(error) = self
            .stabilizer
            .ensure_no_takeover(planning_activity, &cancellation)
        {
            return self.stop_with(&mut run_id, error).await;
        }
        let mut next_turn = response.turn_number;
        let mut completion_has_fresh_turn = false;

        loop {
            if cancellation.is_cancelled() {
                return self.stop_with(&mut run_id, cancelled()).await;
            }
            match response.status {
                PlannerStatus::Completed { message_vi } => {
                    if !completion_has_fresh_turn {
                        return self
                            .stop_with(
                                &mut run_id,
                                protocol_error("completion requires a fresh follow-up observation"),
                            )
                            .await;
                    }
                    run_id.take();
                    let _release = self.executor.release_all();
                    ui.status(AgentState::Completed, &message_vi, Some(&app));
                    return Ok(RuntimeResult::Completed(message_vi));
                }
                PlannerStatus::NeedsUser {
                    reason_code,
                    message_vi,
                    ..
                } => {
                    if let Some(active_run_id) = run_id.take() {
                        self.planner.stop_run(&self.config, &active_run_id).await;
                    }
                    let _release = self.executor.release_all();
                    ui.status(AgentState::NeedsUser, &message_vi, Some(&app));
                    return Ok(RuntimeResult::NeedsUser {
                        reason_code,
                        message_vi,
                        choices: Vec::new(),
                    });
                }
                PlannerStatus::Actions { mut actions } => {
                    if let Err(error) = scope
                        .limits
                        .record_turn(u32::try_from(actions.len()).unwrap_or(u32::MAX))
                    {
                        return self.stop_with(&mut run_id, error).await;
                    }
                    if actions.len() != 1 {
                        return self
                            .stop_with(&mut run_id, protocol_error("one action required"))
                            .await;
                    }
                    let planned = actions.remove(0);
                    ui.status(
                        AgentState::Validating,
                        "Đang kiểm tra ứng dụng, cửa sổ và thao tác…",
                        Some(&app),
                    );
                    let evidence = match self.executor.validate(&app, &observation, &planned) {
                        Ok(evidence) => evidence,
                        Err(error) if error.code == ErrorCode::StaleObservation => {
                            if let Err(error) = scope.limits.record_stale() {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            ui.status(
                                AgentState::StaleRecovery,
                                "Giao diện đã đổi — Tro đang quan sát lại…",
                                Some(&app),
                            );
                            let receipt = stale_receipt(&planned.observation_id);
                            observation = match self.observer.observe(&app, ObservationMode::Full) {
                                Ok(observation) => observation,
                                Err(error) => return self.stop_with(&mut run_id, error).await,
                            };
                            if let Err(error) = scope.bind(observation.metadata.binding.clone()) {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            next_turn = next_turn.saturating_add(1);
                            let planning_activity = self.stabilizer.activity_snapshot();
                            response = tokio::select! {
                                result = self.planner.next_turn(
                                    &self.config,
                                    goal,
                                    run_id.as_deref().unwrap_or_default(),
                                    next_turn,
                                    vec![receipt],
                                    &observation,
                                ) => match result {
                                    Ok(response) => response,
                                    Err(error) => return self.stop_with(&mut run_id, error).await,
                                },
                                error = self.stabilizer.wait_for_takeover(planning_activity, &cancellation) => {
                                    return self.stop_with(&mut run_id, error).await;
                                }
                            };
                            if let Err(error) = self
                                .stabilizer
                                .ensure_no_takeover(planning_activity, &cancellation)
                            {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            completion_has_fresh_turn = true;
                            continue;
                        }
                        Err(error) => return self.stop_with(&mut run_id, error).await,
                    };
                    let decision = ActionPolicy::evaluate(
                        &planned.action,
                        &ActionContext {
                            explicit_session: true,
                            scope_matches: scope.goal_matches(goal)
                                && scope.validates(&observation.metadata.binding),
                            app_approved: scope.is_app_approved(&app.app_id),
                            foreground: &observation.foreground,
                            target: planned.target,
                            evidence: &evidence,
                        },
                    );
                    if decision.tier == RiskTier::Blocked {
                        return self
                            .stop_with(
                                &mut run_id,
                                AppError::new(ErrorCode::ActionBlocked, decision.display_vi, false),
                            )
                            .await;
                    }
                    if decision.tier == RiskTier::Confirm {
                        ui.status(
                            AgentState::AwaitingConfirmation,
                            "Tro đang chờ bạn xác nhận đúng một thao tác.",
                            Some(&app),
                        );
                        let confirmed = match ui
                            .confirm_action(scope.scope_id(), &app, &observation, &planned)
                            .await
                        {
                            Ok(confirmed) => confirmed,
                            Err(error) => return self.stop_with(&mut run_id, error).await,
                        };
                        if !confirmed {
                            return self.stop_with(&mut run_id, cancelled()).await;
                        }
                        // The prompt stole focus. Validation restores the target and
                        // rechecks the complete binding before input.
                        if let Err(error) = self.executor.revalidate_after_confirmation(
                            &app,
                            &observation,
                            &planned,
                        ) {
                            if error.code != ErrorCode::StaleObservation {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            if let Err(error) = scope.limits.record_stale() {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            ui.status(
                                AgentState::StaleRecovery,
                                "Cửa sổ đã đổi sau xác nhận — Tro đang quan sát lại…",
                                Some(&app),
                            );
                            let receipt = stale_receipt(&planned.observation_id);
                            observation = match self.observer.observe(&app, ObservationMode::Full) {
                                Ok(observation) => observation,
                                Err(error) => return self.stop_with(&mut run_id, error).await,
                            };
                            if let Err(error) = scope.bind(observation.metadata.binding.clone()) {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            next_turn = next_turn.saturating_add(1);
                            let planning_activity = self.stabilizer.activity_snapshot();
                            response = tokio::select! {
                                result = self.planner.next_turn(
                                    &self.config,
                                    goal,
                                    run_id.as_deref().unwrap_or_default(),
                                    next_turn,
                                    vec![receipt],
                                    &observation,
                                ) => match result {
                                    Ok(response) => response,
                                    Err(error) => return self.stop_with(&mut run_id, error).await,
                                },
                                error = self.stabilizer.wait_for_takeover(planning_activity, &cancellation) => {
                                    return self.stop_with(&mut run_id, error).await;
                                }
                            };
                            if let Err(error) = self
                                .stabilizer
                                .ensure_no_takeover(planning_activity, &cancellation)
                            {
                                return self.stop_with(&mut run_id, error).await;
                            }
                            completion_has_fresh_turn = true;
                            continue;
                        }
                    }
                    ui.status(AgentState::Executing, &planned.description_vi, Some(&app));
                    let previous_digest = observation.digest();
                    if let Err(error) =
                        self.executor
                            .execute(&app, &observation, &planned, &cancellation)
                    {
                        return self.stop_with(&mut run_id, error).await;
                    }
                    scope.limits.record_execution();
                    ui.status(
                        AgentState::Stabilizing,
                        "Đang chờ ứng dụng ổn định…",
                        Some(&app),
                    );
                    let no_change_allowed = matches!(
                        planned.action,
                        ComputerAction::Move { .. }
                            | ComputerAction::Element {
                                operation: contracts::ElementOperationKind::Focus,
                                ..
                            }
                            | ComputerAction::Wait { .. }
                            | ComputerAction::Capture
                    );
                    observation = match self
                        .stabilizer
                        .wait_for_stable(&app, previous_digest, no_change_allowed, &cancellation)
                        .await
                    {
                        Ok(observation) => observation,
                        Err(error) => return self.stop_with(&mut run_id, error).await,
                    };
                    if let Err(error) = scope.bind(observation.metadata.binding.clone()) {
                        return self.stop_with(&mut run_id, error).await;
                    }
                    ui.status(
                        AgentState::Observing,
                        "Đang xem trạng thái mới nhất…",
                        Some(&app),
                    );
                    let receipt =
                        executed_receipt(&planned.observation_id, &evidence, decision.reason_code);
                    next_turn = next_turn.saturating_add(1);
                    let planning_activity = self.stabilizer.activity_snapshot();
                    response = tokio::select! {
                        result = self.planner.next_turn(
                            &self.config,
                            goal,
                            run_id.as_deref().unwrap_or_default(),
                            next_turn,
                            vec![receipt],
                            &observation,
                        ) => match result {
                            Ok(response) => response,
                            Err(error) => return self.stop_with(&mut run_id, error).await,
                        },
                        error = self.stabilizer.wait_for_takeover(planning_activity, &cancellation) => {
                            return self.stop_with(&mut run_id, error).await;
                        }
                    };
                    if let Err(error) = self
                        .stabilizer
                        .ensure_no_takeover(planning_activity, &cancellation)
                    {
                        return self.stop_with(&mut run_id, error).await;
                    }
                    completion_has_fresh_turn = true;
                }
            }
        }
    }

    async fn stop_with(
        &self,
        run_id: &mut Option<String>,
        error: AppError,
    ) -> Result<RuntimeResult, AppError> {
        if let Some(run_id) = run_id.take() {
            self.planner.stop_run(&self.config, &run_id).await;
        }
        let _release = self.executor.release_all();
        if error.code == ErrorCode::UserTakeover {
            Ok(RuntimeResult::PausedByUser)
        } else {
            Err(error)
        }
    }
}

fn stale_receipt(observation_id: &str) -> ActionReceipt {
    ActionReceipt {
        action_index: 0,
        observation_id: observation_id.to_owned(),
        outcome: ActionOutcome::Stale,
        error_code: Some(ErrorCode::StaleObservation.as_str().to_owned()),
        evidence: ActionReceiptEvidence {
            app_match: false,
            window_match: false,
            resolved_role_category: None,
            policy_reason: Some(PolicyReason::StaleObservation),
        },
        fresh_observation_required: true,
    }
}

fn executed_receipt(
    observation_id: &str,
    evidence: &ResolvedActionEvidence,
    policy_reason: PolicyReason,
) -> ActionReceipt {
    ActionReceipt {
        action_index: 0,
        observation_id: observation_id.to_owned(),
        outcome: ActionOutcome::Executed,
        error_code: None,
        evidence: ActionReceiptEvidence {
            app_match: evidence.app_match,
            window_match: evidence.window_match && evidence.layout_match,
            resolved_role_category: evidence.role_category.clone(),
            policy_reason: Some(policy_reason),
        },
        fresh_observation_required: true,
    }
}

fn cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "Đã dừng computer use.", false)
}

fn protocol_error(source: &str) -> AppError {
    tracing::warn!(
        component = "agent_runtime",
        operation = "provider_response",
        error_code = "provider_protocol_error",
        source
    );
    AppError::new(
        ErrorCode::ProviderProtocolError,
        "Nhà cung cấp trả về kế hoạch computer use không hợp lệ.",
        true,
    )
}
