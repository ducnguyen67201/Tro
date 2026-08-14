use axum::{
    Json,
    extract::{Multipart, Path, State, multipart::Field},
    http::HeaderMap,
};
use contracts::{
    AgentTurnMetadata, AgentTurnResponse, ApiEnvelope, CreateAgentRunMetadata,
    UiObservationMetadata,
};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    error::ApiError,
    services::{agent_loop, device_tokens},
    state::AppState,
};

const MAX_METADATA_BYTES: usize = 262_144;

pub async fn create_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<ApiEnvelope<AgentTurnResponse>>, ApiError> {
    let device_id = authenticated(&state, &headers).await?;
    let (metadata, observation, screenshot) =
        parse_multipart::<CreateAgentRunMetadata>(multipart, state.config.screenshot_max_bytes)
            .await?;
    if metadata.observation != observation {
        return Err(ApiError::invalid("Observation metadata does not match."));
    }
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
    let (metadata, observation, screenshot) =
        parse_multipart::<AgentTurnMetadata>(multipart, state.config.screenshot_max_bytes).await?;
    if metadata.observation != observation {
        return Err(ApiError::invalid("Observation metadata does not match."));
    }
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

async fn parse_multipart<T>(
    mut multipart: Multipart,
    screenshot_limit: usize,
) -> Result<(T, UiObservationMetadata, Zeroizing<Vec<u8>>), ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let mut metadata = None;
    let mut observation = None;
    let mut screenshot = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::invalid("Invalid multipart body."))?
    {
        let field_name = field.name().map(str::to_owned);
        match field_name.as_deref() {
            Some("metadata") if metadata.is_none() => {
                let bytes = read_field_bounded(field, MAX_METADATA_BYTES, "metadata").await?;
                metadata = Some(
                    serde_json::from_slice(&bytes)
                        .map_err(|_| ApiError::invalid("Invalid metadata JSON."))?,
                );
            }
            Some("screenshot") if screenshot.is_none() => {
                screenshot = Some(read_field_bounded(field, screenshot_limit, "screenshot").await?);
            }
            Some("observation") if observation.is_none() => {
                let bytes = read_field_bounded(
                    field,
                    crate::services::computer_provider::MAX_OBSERVATION_BYTES,
                    "observation",
                )
                .await?;
                observation = Some(
                    serde_json::from_slice(&bytes)
                        .map_err(|_| ApiError::invalid("Invalid observation JSON."))?,
                );
            }
            _ => return Err(ApiError::invalid("Unknown multipart field.")),
        }
    }
    Ok((
        metadata.ok_or_else(|| ApiError::invalid("Missing metadata."))?,
        observation.ok_or_else(|| ApiError::invalid("Missing observation."))?,
        screenshot.ok_or_else(|| ApiError::invalid("Missing screenshot."))?,
    ))
}

async fn read_field_bounded(
    mut field: Field<'_>,
    limit: usize,
    field_name: &'static str,
) -> Result<Zeroizing<Vec<u8>>, ApiError> {
    let mut bytes = Zeroizing::new(Vec::new());
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| ApiError::invalid("Invalid multipart field."))?
    {
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err(ApiError::invalid(match field_name {
                "metadata" => "Metadata is too large.",
                "observation" => "Observation is too large.",
                _ => "Screenshot is too large.",
            }));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn envelope<T>(data: T) -> Json<ApiEnvelope<T>> {
    Json(ApiEnvelope {
        data,
        request_id: format!("req_{}", Uuid::new_v4().simple()),
    })
}
