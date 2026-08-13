use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use contracts::{AgentTurnMetadata, AgentTurnResponse, CreateAgentRunMetadata, ErrorCode};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    error::ApiError,
    repositories::{AgentRunRecord, RunStatus},
    state::AppState,
};

pub async fn create(
    state: &AppState,
    device_id: Uuid,
    metadata: CreateAgentRunMetadata,
    screenshot: &[u8],
) -> Result<AgentTurnResponse, ApiError> {
    check_media(state, &metadata.frame, screenshot)?;
    check_budget(state, device_id).await?;
    let provider = state
        .provider
        .agent_turn(&metadata.goal, screenshot, None)
        .await?;
    if provider.actions.len() > 20 {
        return Err(ApiError::invalid("Provider returned too many actions."));
    }
    let run_id = Uuid::new_v4();
    let response = AgentTurnResponse {
        run_id: run_id.to_string(),
        turn_number: 0,
        actions: provider.actions,
        completed: provider.completed,
    };
    let record = AgentRunRecord {
        id: run_id,
        device_id,
        continuation_encrypted: encrypt_continuation(state, &provider.continuation_id)?,
        status: if response.completed {
            RunStatus::Completed
        } else {
            RunStatus::Active
        },
        turn_count: 1,
        action_count: u32::try_from(response.actions.len()).unwrap_or(u32::MAX),
        expires_at: OffsetDateTime::now_utc()
            + Duration::seconds(
                i64::try_from(state.config.agent_max_seconds.min(1800)).unwrap_or(300),
            ),
        last_idempotency_key: None,
        last_response: None,
    };
    state
        .repository
        .create_run(record)
        .await
        .map_err(|_| ApiError::provider())?;
    state
        .repository
        .increment_agent_usage(device_id)
        .await
        .map_err(|_| ApiError::provider())?;
    Ok(response)
}

pub async fn turn(
    state: &AppState,
    device_id: Uuid,
    run_id: Uuid,
    idempotency_key: &str,
    metadata: AgentTurnMetadata,
    screenshot: &[u8],
) -> Result<AgentTurnResponse, ApiError> {
    check_media(state, &metadata.frame, screenshot)?;
    let mut run = owned_active_run(state, device_id, run_id).await?;
    if run.last_idempotency_key.as_deref() == Some(idempotency_key) {
        let cached = run
            .last_response
            .as_deref()
            .ok_or_else(|| ApiError::invalid("Idempotency state is incomplete."))?;
        return serde_json::from_slice(cached)
            .map_err(|_| ApiError::invalid("Cached response is invalid."));
    }
    if metadata.turn_number != run.turn_count {
        return Err(ApiError::invalid("Turn number is stale or out of order."));
    }
    if run.turn_count >= state.config.agent_max_turns {
        return Err(limit_error(
            ErrorCode::AgentTurnLimit,
            "Phiên đã đạt giới hạn lượt.",
        ));
    }
    check_budget(state, device_id).await?;
    let previous = decrypt_continuation(state, &run.continuation_encrypted)?;
    let provider = state
        .provider
        .agent_turn(
            "Continue the immutable declared goal.",
            screenshot,
            Some(&previous),
        )
        .await?;
    let new_action_count = run
        .action_count
        .saturating_add(u32::try_from(provider.actions.len()).unwrap_or(u32::MAX));
    if new_action_count > state.config.agent_max_actions {
        return Err(limit_error(
            ErrorCode::AgentTurnLimit,
            "Phiên đã đạt giới hạn thao tác.",
        ));
    }
    let response = AgentTurnResponse {
        run_id: run_id.to_string(),
        turn_number: metadata.turn_number,
        actions: provider.actions,
        completed: provider.completed,
    };
    run.continuation_encrypted = encrypt_continuation(state, &provider.continuation_id)?;
    run.turn_count = run.turn_count.saturating_add(1);
    run.action_count = new_action_count;
    run.status = if response.completed {
        RunStatus::Completed
    } else {
        RunStatus::Active
    };
    run.last_idempotency_key = Some(idempotency_key.to_owned());
    run.last_response = Some(serde_json::to_vec(&response).map_err(|_| ApiError::provider())?);
    state
        .repository
        .update_run(run)
        .await
        .map_err(|_| ApiError::provider())?;
    state
        .repository
        .increment_agent_usage(device_id)
        .await
        .map_err(|_| ApiError::provider())?;
    Ok(response)
}

pub async fn stop(state: &AppState, device_id: Uuid, run_id: Uuid) -> Result<(), ApiError> {
    let mut run = state
        .repository
        .get_run(run_id)
        .await
        .map_err(|_| ApiError::provider())?
        .ok_or_else(|| ApiError::invalid("Run not found."))?;
    if run.device_id != device_id {
        return Err(ApiError::unauthorized());
    }
    run.status = RunStatus::Stopped;
    run.continuation_encrypted.clear();
    state
        .repository
        .update_run(run)
        .await
        .map_err(|_| ApiError::provider())
}

async fn owned_active_run(
    state: &AppState,
    device_id: Uuid,
    run_id: Uuid,
) -> Result<AgentRunRecord, ApiError> {
    let run = state
        .repository
        .get_run(run_id)
        .await
        .map_err(|_| ApiError::provider())?
        .ok_or_else(|| ApiError::invalid("Run not found."))?;
    if run.device_id != device_id {
        return Err(ApiError::unauthorized());
    }
    if run.status != RunStatus::Active || run.expires_at <= OffsetDateTime::now_utc() {
        return Err(ApiError::invalid("Run is stopped, complete, or expired."));
    }
    Ok(run)
}

async fn check_budget(state: &AppState, device_id: Uuid) -> Result<(), ApiError> {
    if !state.config.agent_enabled {
        return Err(ApiError::disabled("agent"));
    }
    let usage = state
        .repository
        .usage_today(device_id)
        .await
        .map_err(|_| ApiError::provider())?;
    if usage.agent_turns >= state.config.device_daily_agent_turns
        || usage.screenshots >= state.config.device_daily_screenshots
    {
        return Err(limit_error(
            ErrorCode::RateLimited,
            "Bạn đã đạt giới hạn agent hôm nay.",
        ));
    }
    Ok(())
}

fn check_media(
    state: &AppState,
    frame: &contracts::ScreenFrameMeta,
    screenshot: &[u8],
) -> Result<(), ApiError> {
    if screenshot.is_empty()
        || screenshot.len() > state.config.screenshot_max_bytes
        || frame.width_px > state.config.screenshot_max_edge_px
        || frame.height_px > state.config.screenshot_max_edge_px
    {
        return Err(ApiError::invalid("Ảnh màn hình vượt giới hạn an toàn."));
    }
    Ok(())
}

fn cipher(state: &AppState) -> Result<Aes256Gcm, ApiError> {
    Aes256Gcm::new_from_slice(state.config.agent_continuation_aead_key.expose().as_bytes())
        .map_err(|_| ApiError::provider())
}

fn encrypt_continuation(state: &AppState, value: &str) -> Result<Vec<u8>, ApiError> {
    let mut nonce = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let encrypted = cipher(state)?
        .encrypt(Nonce::from_slice(&nonce), value.as_bytes())
        .map_err(|_| ApiError::provider())?;
    Ok([nonce.as_slice(), encrypted.as_slice()].concat())
}

fn decrypt_continuation(state: &AppState, value: &[u8]) -> Result<String, ApiError> {
    let (nonce, encrypted) = value
        .split_at_checked(12)
        .ok_or_else(|| ApiError::invalid("Continuation is invalid."))?;
    let plain = cipher(state)?
        .decrypt(Nonce::from_slice(nonce), encrypted)
        .map_err(|_| ApiError::provider())?;
    String::from_utf8(plain).map_err(|_| ApiError::provider())
}

fn limit_error(code: ErrorCode, message: &str) -> ApiError {
    ApiError {
        status: axum::http::StatusCode::TOO_MANY_REQUESTS,
        app: contracts::AppError::new(code, message, false),
    }
}
