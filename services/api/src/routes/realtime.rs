use axum::{Json, extract::State, http::HeaderMap};
use contracts::{ApiEnvelope, RealtimeSecretRequest, RealtimeSecretResponse};
use uuid::Uuid;

use crate::{
    error::ApiError,
    services::{device_tokens, realtime_tokens},
    state::AppState,
};

pub async fn client_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RealtimeSecretRequest>,
) -> Result<Json<ApiEnvelope<RealtimeSecretResponse>>, ApiError> {
    let bearer = device_tokens::bearer_value(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    )?;
    let device_id = device_tokens::authenticate(&state, bearer).await?;
    let response = realtime_tokens::mint(&state, device_id, request).await?;
    Ok(Json(ApiEnvelope {
        data: response,
        request_id: format!("req_{}", Uuid::new_v4().simple()),
    }))
}
