use contracts::{TelemetryBatch, TelemetryEvent};
use std::collections::BTreeMap;

#[test]
fn rejects_content_bearing_telemetry() {
    let batch = TelemetryBatch {
        events: vec![TelemetryEvent {
            name: "stable_error".to_owned(),
            occurred_at_unix: 1,
            attributes: BTreeMap::from([("transcript".to_owned(), "private fixture".to_owned())]),
        }],
    };
    assert!(api::services::telemetry::validate(&batch).is_err());
}

#[test]
fn rejects_non_allowlisted_labels_even_without_a_forbidden_key_name() {
    let batch = TelemetryBatch {
        events: vec![TelemetryEvent {
            name: "policy_decision".to_owned(),
            occurred_at_unix: 1,
            attributes: BTreeMap::from([("label".to_owned(), "Hoa Tui".to_owned())]),
        }],
    };
    assert!(api::services::telemetry::validate(&batch).is_err());
}

#[test]
fn accepts_content_free_computer_use_diagnostics() {
    let batch = TelemetryBatch {
        events: vec![TelemetryEvent {
            name: "latency_bucket".to_owned(),
            occurred_at_unix: 1,
            attributes: BTreeMap::from([
                ("provider_kind".to_owned(), "scale_cua".to_owned()),
                ("model".to_owned(), "scalecua".to_owned()),
                ("semantic_mode".to_owned(), "ax".to_owned()),
                ("element_count".to_owned(), "42".to_owned()),
                ("truncated".to_owned(), "false".to_owned()),
                ("action_kind".to_owned(), "element".to_owned()),
                ("outcome".to_owned(), "executed".to_owned()),
                ("confirmation_tier".to_owned(), "confirm".to_owned()),
            ]),
        }],
    };
    api::services::telemetry::validate(&batch).expect("content-free diagnostics are allowlisted");
}
