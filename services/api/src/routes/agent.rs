use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::HeaderMap,
};
use contracts::{AgentTurnMetadata, AgentTurnResponse, ApiEnvelope, CreateAgentRunMetadata};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    error::ApiError,
    services::{agent_loop, device_tokens},
    state::AppState,
};

pub async fn create_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<ApiEnvelope<AgentTurnResponse>>, ApiError> {
    let device_id = authenticated(&state, &headers).await?;
    let (metadata, screenshot) = parse_multipart::<CreateAgentRunMetadata>(multipart).await?;
    let response = agent_loop::create(&state, device_id, metadata, &screenshot).await?;
    Ok(envelope(response))
}

pub async fn next_turn(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<ApiEnvelope<AgentTurnResponse>>, ApiError> {
    let device_id = authenticated(&state, &headers).await?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .ok_or_else(|| ApiError::invalid("Missing idempotency key."))?;
    let (metadata, screenshot) = parse_multipart::<AgentTurnMetadata>(multipart).await?;
    let response = agent_loop::turn(
        &state,
        device_id,
        id,
        idempotency_key,
        metadata,
        &screenshot,
    )
    .await?;
    Ok(envelope(response))
}

pub async fn stop_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, ApiError> {
    let device_id = authenticated(&state, &headers).await?;
    agent_loop::stop(&state, device_id, id).await?;
    Ok(envelope(serde_json::json!({"stopped": true})))
}

async fn authenticated(state: &AppState, headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let bearer = device_tokens::bearer_value(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    )?;
    device_tokens::authenticate(state, bearer).await
}

async fn parse_multipart<T>(mut multipart: Multipart) -> Result<(T, Zeroizing<Vec<u8>>), ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let mut metadata = None;
    let mut screenshot = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::invalid("Invalid multipart body."))?
    {
        match field.name() {
            Some("metadata") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| ApiError::invalid("Invalid metadata."))?;
                metadata = Some(
                    serde_json::from_slice(&bytes)
                        .map_err(|_| ApiError::invalid("Invalid metadata JSON."))?,
                );
            }
            Some("screenshot") => {
                screenshot = Some(Zeroizing::new(
                    field
                        .bytes()
                        .await
                        .map_err(|_| ApiError::invalid("Invalid screenshot."))?
                        .to_vec(),
                ));
            }
            _ => return Err(ApiError::invalid("Unknown multipart field.")),
        }
    }
    Ok((
        metadata.ok_or_else(|| ApiError::invalid("Missing metadata."))?,
        screenshot.ok_or_else(|| ApiError::invalid("Missing screenshot."))?,
    ))
}

fn envelope<T>(data: T) -> Json<ApiEnvelope<T>> {
    Json(ApiEnvelope {
        data,
        request_id: format!("req_{}", Uuid::new_v4().simple()),
    })
}
