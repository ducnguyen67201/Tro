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
