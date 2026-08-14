use axum::{Json, extract::State};
use contracts::{
    ApiEnvelope, DeviceTokenResponse, GoogleAuthCompleteRequest, GoogleAuthStartRequest,
    GoogleAuthStartResponse,
};
use uuid::Uuid;

use crate::{
    error::ApiError,
    services::google_auth::{complete_google_login, start_google_login},
    state::AppState,
};

pub async fn google_start(
    State(state): State<AppState>,
    Json(request): Json<GoogleAuthStartRequest>,
) -> Result<Json<ApiEnvelope<GoogleAuthStartResponse>>, ApiError> {
    let response = start_google_login(&state.config, &request)?;
    Ok(Json(ApiEnvelope {
        data: response,
        request_id: request_id(),
    }))
}

pub async fn google_complete(
    State(state): State<AppState>,
    Json(request): Json<GoogleAuthCompleteRequest>,
) -> Result<Json<ApiEnvelope<DeviceTokenResponse>>, ApiError> {
    let response = complete_google_login(&state, request).await?;
    Ok(Json(ApiEnvelope {
        data: response,
        request_id: request_id(),
    }))
}

fn request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}
