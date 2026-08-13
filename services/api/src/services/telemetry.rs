use contracts::TelemetryBatch;

use crate::error::ApiError;

const ALLOWED_NAMES: &[&str] = &[
    "session_started",
    "session_stopped",
    "permission_status",
    "latency_bucket",
    "policy_decision",
    "stable_error",
];
const FORBIDDEN_KEYS: &[&str] = &[
    "text",
    "transcript",
    "prompt",
    "screenshot",
    "audio",
    "token",
    "authorization",
    "coordinates",
    "window_title",
];

pub fn validate(batch: &TelemetryBatch) -> Result<(), ApiError> {
    if batch.events.len() > 100 {
        return Err(ApiError::invalid("Telemetry batch is too large."));
    }
    for event in &batch.events {
        if !ALLOWED_NAMES.contains(&event.name.as_str()) || event.attributes.len() > 16 {
            return Err(ApiError::invalid("Telemetry event is not allowlisted."));
        }
        for (key, value) in &event.attributes {
            let normalized = key.to_ascii_lowercase();
            if FORBIDDEN_KEYS.iter().any(|item| normalized.contains(item)) || value.len() > 128 {
                return Err(ApiError::invalid("Telemetry contains forbidden content."));
            }
        }
    }
    Ok(())
}
