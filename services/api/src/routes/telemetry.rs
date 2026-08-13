use axum::{Json, extract::State, http::HeaderMap};
use contracts::{ApiEnvelope, TelemetryBatch};
use uuid::Uuid;

use crate::{
    error::ApiError,
    services::{device_tokens, telemetry},
    state::AppState,
};

pub async fn batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(batch): Json<TelemetryBatch>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, ApiError> {
    let bearer = device_tokens::bearer_value(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    )?;
    let _device_id = device_tokens::authenticate(&state, bearer).await?;
    telemetry::validate(&batch)?;
    Ok(Json(ApiEnvelope {
        data: serde_json::json!({"accepted": batch.events.len()}),
        request_id: format!("req_{}", Uuid::new_v4().simple()),
    }))
}
