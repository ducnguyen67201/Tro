use contracts::{
    ActionLocator, AgentState, AssistantState, CaptureScope, ComputerAction, ElementOperationKind,
    NormalizedPoint, ObservationBinding, OverlayElement, PlannedComputerAction, SecretText,
    UiElementSnapshot, UiObservationMetadata, UiState,
};

#[test]
fn contract_fixture_round_trips() {
    let fixture = include_str!("fixtures/contracts.json");
    let value: serde_json::Value = serde_json::from_str(fixture).expect("fixture is static JSON");
    let encoded = serde_json::to_string(&value).expect("JSON value serializes");
    let decoded: serde_json::Value = serde_json::from_str(&encoded).expect("encoded JSON parses");
    assert_eq!(value, decoded);
}

#[test]
fn app_scoped_action_and_observation_round_trip() {
    let observation = UiObservationMetadata {
        binding: ObservationBinding {
            observation_id: "obs-1".to_owned(),
            app_id: "app-1".to_owned(),
            window_generation: 4,
            layout_generation: 2,
        },
        capture_scope: CaptureScope::ExactWindow,
        elements: vec![UiElementSnapshot {
            element_id: "e_0".to_owned(),
            role: SecretText::new("button"),
            name: Some(SecretText::new("Course 5")),
            value: None,
            bounds: None,
            states: vec![UiState::Enabled],
            operations: vec![ElementOperationKind::Invoke],
            children: Vec::new(),
        }],
        truncated: false,
    };
    let action = PlannedComputerAction {
        observation_id: "obs-1".to_owned(),
        locator: ActionLocator::Element {
            element_id: "e_0".to_owned(),
        },
        action: ComputerAction::Element {
            operation: ElementOperationKind::Invoke,
            value: None,
        },
        target: contracts::ActionTarget::Benign,
        description_vi: "Mở khóa học số năm".to_owned(),
    };
    let encoded =
        serde_json::to_vec(&(observation.clone(), action.clone())).expect("contracts serialize");
    let decoded: (UiObservationMetadata, PlannedComputerAction) =
        serde_json::from_slice(&encoded).expect("contracts deserialize");
    assert_eq!(decoded, (observation, action));
    assert!(!format!("{:?}", decoded.0.elements[0]).contains("Course 5"));
}

#[test]
fn canonical_actions_are_tagged_and_bounded() {
    let action = ComputerAction::TypeText {
        text: SecretText::new("Tiếng Việt"),
    };
    let json = serde_json::to_string(&action).expect("action serializes");
    assert!(json.contains("type_text"));
    assert!(!format!("{action:?}").contains("Tiếng Việt"));

    let point = NormalizedPoint::new(0.25, 0.75).expect("point is valid");
    let overlay = OverlayElement::Point {
        at: point,
        label: Some("Bấm vào đây".to_owned()),
    };
    assert!(serde_json::to_value(overlay).is_ok());
    assert_eq!(AssistantState::default(), AssistantState::Idle);
    assert_eq!(AgentState::default(), AgentState::Idle);
}
