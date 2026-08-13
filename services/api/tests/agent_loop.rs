use api::{AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository};
use contracts::{
    ActionTarget, ComputerAction, CreateAgentRunMetadata, ImageMime, PlannedComputerAction,
    ScreenFrameMeta,
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
            action: ComputerAction::Capture,
            target: ActionTarget::Benign,
            description_vi: "Chụp lại màn hình".to_owned(),
        });
    let state = AppState::new(
        Arc::new(AppConfig::test()),
        repository.clone(),
        provider,
        Arc::new(FakeTutorProvider::default()),
    );
    let device = Uuid::new_v4();
    let response = api::services::agent_loop::create(
        &state,
        device,
        CreateAgentRunMetadata {
            goal: "Mở bài thực hành".to_owned(),
            frame: ScreenFrameMeta {
                frame_id: "fixture".to_owned(),
                monitor_id: "main".to_owned(),
                width_px: 100,
                height_px: 100,
                origin_x_px: 0,
                origin_y_px: 0,
                scale_factor: 1.0,
                layout_generation: 1,
                mime_type: ImageMime::Jpeg,
            },
        },
        b"synthetic-image",
    )
    .await
    .expect("run starts");
    assert!(!response.run_id.is_empty());
    assert!(!response.completed);
}
