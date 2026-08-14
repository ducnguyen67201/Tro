use api::{AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository, Repository};
use contracts::{
    ActionLocator, ActionOutcome, ActionReceipt, ActionReceiptEvidence, ActionTarget,
    ApplicationRef, CaptureScope, ComputerAction, CreateAgentRunMetadata, ImageMime,
    ObservationBinding, PlannedComputerAction, PlannerStatus, PolicyReason, ScreenFrameMeta,
    UiObservationMetadata,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn run_stores_only_encrypted_continuation_metadata() {
    let repository = Arc::new(MemoryRepository::default());
    let provider = Arc::new(FakeProvider::default());
    provider
        .actions
        .lock()
        .expect("fake provider mutex")
        .push(PlannedComputerAction {
            observation_id: "obs-1".to_owned(),
            locator: ActionLocator::Frame,
            action: ComputerAction::Capture,
            target: ActionTarget::Benign,
            description_vi: "Chụp lại màn hình".to_owned(),
        });
    let state = AppState::new(
        Arc::new(AppConfig::test()),
        repository.clone(),
        provider.clone(),
        Arc::new(FakeTutorProvider::default()),
    );
    let device = Uuid::new_v4();
    let observation = UiObservationMetadata {
        binding: ObservationBinding {
            observation_id: "obs-1".to_owned(),
            app_id: "browser".to_owned(),
            window_generation: 1,
            layout_generation: 1,
        },
        capture_scope: CaptureScope::ExactWindow,
        elements: Vec::new(),
        truncated: false,
    };
    let frame = ScreenFrameMeta {
        frame_id: "fixture".to_owned(),
        monitor_id: "main".to_owned(),
        width_px: 100,
        height_px: 100,
        image_width_px: 100,
        image_height_px: 100,
        origin_x_px: 0,
        origin_y_px: 0,
        scale_factor: 1.0,
        layout_generation: 1,
        mime_type: ImageMime::Jpeg,
    };
    let jpeg = [0xff, 0xd8, 0xff, 0xd9];
    let response = api::services::agent_loop::create(
        &state,
        device,
        CreateAgentRunMetadata {
            goal: "Mở bài thực hành".to_owned(),
            frame: frame.clone(),
            observation: observation.clone(),
            available_apps: vec![ApplicationRef {
                app_id: "browser".to_owned(),
                display_name: "ABC Browser".to_owned(),
                identity_summary: "fixture".to_owned(),
            }],
        },
        &jpeg,
    )
    .await
    .expect("run starts");
    assert!(!response.run_id.is_empty());
    assert!(matches!(response.status, PlannerStatus::Actions { .. }));
    let run_id = Uuid::parse_str(&response.run_id).expect("run ID");
    let stored = repository
        .get_run(run_id)
        .await
        .expect("repository read")
        .expect("stored run");
    let encrypted_text = String::from_utf8_lossy(&stored.continuation_encrypted);
    assert!(!encrypted_text.contains("Mở bài thực hành"));

    provider
        .actions
        .lock()
        .expect("fake provider mutex")
        .clear();
    let receipt = ActionReceipt {
        action_index: 0,
        observation_id: "obs-1".to_owned(),
        outcome: ActionOutcome::Stale,
        error_code: Some("stale_observation".to_owned()),
        evidence: ActionReceiptEvidence {
            app_match: true,
            window_match: false,
            resolved_role_category: None,
            policy_reason: Some(PolicyReason::StaleObservation),
        },
        fresh_observation_required: true,
    };
    let next_observation = UiObservationMetadata {
        binding: ObservationBinding {
            observation_id: "obs-2".to_owned(),
            ..observation.binding
        },
        ..observation
    };
    let changed_goal = api::services::agent_loop::turn(
        &state,
        device,
        run_id,
        "idem-changed-goal",
        contracts::AgentTurnMetadata {
            goal: "Mở một ứng dụng khác".to_owned(),
            turn_number: 1,
            frame: frame.clone(),
            observation: next_observation.clone(),
            receipts: vec![receipt.clone()],
        },
        &jpeg,
    )
    .await;
    assert!(changed_goal.is_err());

    let completed = api::services::agent_loop::turn(
        &state,
        device,
        run_id,
        "idem-1",
        contracts::AgentTurnMetadata {
            goal: "Mở bài thực hành".to_owned(),
            turn_number: 1,
            frame,
            observation: next_observation,
            receipts: vec![receipt.clone()],
        },
        &jpeg,
    )
    .await
    .expect("next turn");
    assert!(matches!(completed.status, PlannerStatus::Completed { .. }));
    let requests = provider.requests.lock().expect("fake request mutex");
    assert_eq!(requests[1].goal, "Mở bài thực hành");
    assert_eq!(requests[1].receipts, vec![receipt]);
    assert!(requests[1].has_continuation);
}
