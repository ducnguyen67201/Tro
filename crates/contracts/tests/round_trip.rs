use contracts::{
    AgentState, AssistantState, ComputerAction, NormalizedPoint, OverlayElement, SecretText,
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
