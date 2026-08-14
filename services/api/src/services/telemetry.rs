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
const ALLOWED_KEYS: &[&str] = &[
    "action_kind",
    "app_version",
    "confirmation_tier",
    "degradation_code",
    "element_count",
    "error_code",
    "latency_bucket",
    "model",
    "outcome",
    "permission",
    "platform",
    "policy_reason",
    "provider_kind",
    "semantic_mode",
    "stale_count",
    "status",
    "truncated",
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
            if !ALLOWED_KEYS.contains(&normalized.as_str())
                || FORBIDDEN_KEYS.iter().any(|item| normalized.contains(item))
                || value.len() > 128
            {
                return Err(ApiError::invalid("Telemetry contains forbidden content."));
            }
        }
    }
    Ok(())
}
