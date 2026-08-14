use contracts::{ActionLocator, ComputerAction, OverlayElement, PlannerStatus};

#[test]
fn schemas_have_closed_action_tags() {
    let action = schemars::schema_for!(ComputerAction);
    let overlay = schemars::schema_for!(OverlayElement);
    let action_json = serde_json::to_string(&action).expect("schema serializes");
    let overlay_json = serde_json::to_string(&overlay).expect("schema serializes");
    assert!(action_json.contains("unsupported") || action_json.contains("kind"));
    assert!(overlay_json.contains("kind"));
    let locator_json = serde_json::to_string(&schemars::schema_for!(ActionLocator))
        .expect("locator schema serializes");
    let status_json = serde_json::to_string(&schemars::schema_for!(PlannerStatus))
        .expect("planner schema serializes");
    assert!(locator_json.contains("element_id"));
    assert!(status_json.contains("needs_user"));
}
