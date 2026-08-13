use axum::{Json, extract::State, http::HeaderMap};
use contracts::{ApiEnvelope, DeviceTokenResponse, RegisterDeviceRequest};
use uuid::Uuid;

use crate::{
    error::ApiError,
    services::device_tokens::{authenticate, bearer_value, register_device, rotate_token},
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterDeviceRequest>,
) -> Result<Json<ApiEnvelope<DeviceTokenResponse>>, ApiError> {
    let response = register_device(&state, request).await?;
    Ok(Json(ApiEnvelope {
        data: response,
        request_id: request_id(),
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<DeviceTokenResponse>>, ApiError> {
    let bearer = bearer_value(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    )?;
    let device_id = authenticate(&state, bearer).await?;
    let response = rotate_token(&state, device_id, bearer).await?;
    Ok(Json(ApiEnvelope {
        data: response,
        request_id: request_id(),
    }))
}

fn request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}
