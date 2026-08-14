use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use contracts::{
    AgentTurnMetadata, AgentTurnResponse, CreateAgentRunMetadata, ErrorCode, PlannerStatus,
    UiObservationMetadata,
};
use std::collections::HashSet;
use std::time::Instant;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    error::ApiError,
    repositories::{AgentRunRecord, RunStatus},
    services::computer_provider::{
        ComputerProviderRequest, MAX_OBSERVATION_BYTES, MAX_OBSERVATION_ELEMENTS, validate_binding,
    },
    state::AppState,
};

pub async fn create(
    state: &AppState,
    device_id: Uuid,
    metadata: CreateAgentRunMetadata,
    screenshot: &[u8],
) -> Result<AgentTurnResponse, ApiError> {
    check_enabled(state)?;
    check_goal(&metadata.goal)?;
    check_available_apps(&metadata.available_apps)?;
    check_media(state, &metadata.frame, screenshot)?;
    check_observation(&metadata.observation, &metadata.frame)?;
    check_budget(state, device_id).await?;
    let request = ComputerProviderRequest {
        goal: &metadata.goal,
        turn_number: 0,
        observation: &metadata.observation,
        available_apps: &metadata.available_apps,
        receipts: &[],
        screenshot,
        screenshot_mime: metadata.frame.mime_type,
        continuation: None,
    };
    let provider_started = Instant::now();
    let provider = state.computer_provider.turn(request).await?;
    validate_provider_status(
        &provider.status,
        &ComputerProviderRequest {
            goal: &metadata.goal,
            turn_number: 0,
            observation: &metadata.observation,
            available_apps: &metadata.available_apps,
            receipts: &[],
            screenshot,
            screenshot_mime: metadata.frame.mime_type,
            continuation: None,
        },
    )?;
    record_provider_metadata(
        provider.provider_kind,
        &provider.model,
        provider_started.elapsed(),
        &provider.status,
    );
    let run_id = Uuid::new_v4();
    let action_count = status_action_count(&provider.status);
    let terminal = !matches!(provider.status, PlannerStatus::Actions { .. });
    let response = AgentTurnResponse {
        run_id: run_id.to_string(),
        turn_number: 0,
        status: provider.status,
    };
    let record = AgentRunRecord {
        id: run_id,
        device_id,
        continuation_encrypted: if terminal {
            Vec::new()
        } else {
            encrypt_continuation(state, &provider.continuation)?
        },
        status: if terminal {
            RunStatus::Completed
        } else {
            RunStatus::Active
        },
        turn_count: 1,
        action_count,
        expires_at: OffsetDateTime::now_utc()
            + Duration::seconds(
                i64::try_from(state.config.agent_max_seconds.min(1_800)).unwrap_or(300),
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
    check_enabled(state)?;
    check_goal(&metadata.goal)?;
    check_receipts(&metadata.receipts)?;
    check_media(state, &metadata.frame, screenshot)?;
    check_observation(&metadata.observation, &metadata.frame)?;
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
    validate_continuation_goal(&previous, &metadata.goal)?;
    let request = ComputerProviderRequest {
        goal: &metadata.goal,
        turn_number: metadata.turn_number,
        observation: &metadata.observation,
        available_apps: &[],
        receipts: &metadata.receipts,
        screenshot,
        screenshot_mime: metadata.frame.mime_type,
        continuation: Some(&previous),
    };
    let provider_started = Instant::now();
    let provider = state.computer_provider.turn(request).await?;
    validate_provider_status(
        &provider.status,
        &ComputerProviderRequest {
            goal: &metadata.goal,
            turn_number: metadata.turn_number,
            observation: &metadata.observation,
            available_apps: &[],
            receipts: &metadata.receipts,
            screenshot,
            screenshot_mime: metadata.frame.mime_type,
            continuation: Some(&previous),
        },
    )?;
    record_provider_metadata(
        provider.provider_kind,
        &provider.model,
        provider_started.elapsed(),
        &provider.status,
    );
    let new_action_count = run
        .action_count
        .saturating_add(status_action_count(&provider.status));
    if new_action_count > state.config.agent_max_actions {
        return Err(limit_error(
            ErrorCode::AgentTurnLimit,
            "Phiên đã đạt giới hạn thao tác.",
        ));
    }
    let terminal = !matches!(provider.status, PlannerStatus::Actions { .. });
    let response = AgentTurnResponse {
        run_id: run_id.to_string(),
        turn_number: metadata.turn_number,
        status: provider.status,
    };
    run.continuation_encrypted = if terminal {
        Vec::new()
    } else {
        encrypt_continuation(state, &provider.continuation)?
    };
    run.turn_count = run.turn_count.saturating_add(1);
    run.action_count = new_action_count;
    run.status = if terminal {
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
    run.last_response = None;
    run.last_idempotency_key = None;
    state
        .repository
        .update_run(run)
        .await
        .map_err(|_| ApiError::provider())
}

fn validate_provider_status(
    status: &PlannerStatus,
    request: &ComputerProviderRequest<'_>,
) -> Result<(), ApiError> {
    match status {
        PlannerStatus::Actions { actions } if actions.len() == 1 => {
            validate_binding(&actions[0], request)
        }
        PlannerStatus::Actions { .. } => Err(ApiError::invalid(
            "Provider must return exactly one bound action.",
        )),
        PlannerStatus::Completed { message_vi } if !message_vi.trim().is_empty() => Ok(()),
        PlannerStatus::NeedsUser { message_vi, .. } if !message_vi.trim().is_empty() => Ok(()),
        _ => Err(ApiError::invalid("Provider status is incomplete.")),
    }
}

fn status_action_count(status: &PlannerStatus) -> u32 {
    match status {
        PlannerStatus::Actions { actions } => u32::try_from(actions.len()).unwrap_or(u32::MAX),
        PlannerStatus::Completed { .. } | PlannerStatus::NeedsUser { .. } => 0,
    }
}

fn validate_continuation_goal(value: &str, goal: &str) -> Result<(), ApiError> {
    if !(3..=500).contains(&goal.len()) {
        return Err(ApiError::invalid("Goal is outside safe bounds."));
    }
    let parsed: serde_json::Value =
        serde_json::from_str(value).map_err(|_| ApiError::invalid("Continuation is invalid."))?;
    let expected = parsed
        .get("goal_hash")
        .and_then(serde_json::Value::as_str)
        .filter(|hash| hash.len() == 64)
        .ok_or_else(|| ApiError::invalid("Continuation has no immutable goal binding."))?;
    if expected != blake3::hash(goal.as_bytes()).to_hex().as_str() {
        return Err(ApiError::invalid("Goal changed during the run."));
    }
    Ok(())
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

fn check_enabled(state: &AppState) -> Result<(), ApiError> {
    if !state.config.agent_enabled || !state.config.reliable_computer_use_enabled {
        return Err(ApiError::disabled("reliable_computer_use"));
    }
    Ok(())
}

fn check_media(
    state: &AppState,
    frame: &contracts::ScreenFrameMeta,
    screenshot: &[u8],
) -> Result<(), ApiError> {
    let mime_matches = match frame.mime_type {
        contracts::ImageMime::Jpeg => screenshot.starts_with(&[0xff, 0xd8, 0xff]),
        contracts::ImageMime::Png => screenshot.starts_with(b"\x89PNG\r\n\x1a\n"),
    };
    if screenshot.is_empty()
        || screenshot.len() > state.config.screenshot_max_bytes
        || frame.image_width_px > state.config.screenshot_max_edge_px
        || frame.image_height_px > state.config.screenshot_max_edge_px
        || frame.image_width_px == 0
        || frame.image_height_px == 0
        || frame.width_px == 0
        || frame.height_px == 0
        || !mime_matches
    {
        return Err(ApiError::invalid("Ảnh màn hình vượt giới hạn an toàn."));
    }
    Ok(())
}

fn check_observation(
    observation: &UiObservationMetadata,
    frame: &contracts::ScreenFrameMeta,
) -> Result<(), ApiError> {
    let bytes = serde_json::to_vec(observation)
        .map_err(|_| ApiError::invalid("Observation is invalid."))?;
    let mut element_ids = HashSet::new();
    let elements_valid = observation.elements.iter().all(|element| {
        !element.element_id.is_empty()
            && element.element_id.len() <= 64
            && element_ids.insert(element.element_id.as_str())
            && element.role.expose().len() <= 128
            && element
                .name
                .as_ref()
                .is_none_or(|value| value.expose().len() <= 512)
            && element
                .value
                .as_ref()
                .is_none_or(|value| value.expose().len() <= 2_000)
            && element.states.len() <= 16
            && element.operations.len() <= 16
            && element.children.len() <= 64
            && element
                .children
                .iter()
                .all(|child| !child.is_empty() && child.len() <= 64)
    });
    let children_known = observation.elements.iter().all(|element| {
        element
            .children
            .iter()
            .all(|child| element_ids.contains(child.as_str()))
    });
    if bytes.len() > MAX_OBSERVATION_BYTES
        || observation.elements.len() > MAX_OBSERVATION_ELEMENTS
        || observation.binding.observation_id.is_empty()
        || observation.binding.observation_id.len() > 64
        || observation.binding.app_id.is_empty()
        || observation.binding.app_id.len() > 200
        || observation.binding.layout_generation != frame.layout_generation
        || !elements_valid
        || !children_known
    {
        return Err(ApiError::invalid("Observation exceeds safe bounds."));
    }
    Ok(())
}

fn check_goal(goal: &str) -> Result<(), ApiError> {
    if (3..=500).contains(&goal.len()) {
        Ok(())
    } else {
        Err(ApiError::invalid("Goal is outside safe bounds."))
    }
}

fn check_available_apps(apps: &[contracts::ApplicationRef]) -> Result<(), ApiError> {
    if apps.len() <= 256
        && apps.iter().all(|app| {
            !app.app_id.is_empty()
                && app.app_id.len() <= 200
                && !app.display_name.is_empty()
                && app.display_name.len() <= 256
                && app.identity_summary.len() <= 256
        })
    {
        Ok(())
    } else {
        Err(ApiError::invalid(
            "Application catalog exceeds safe bounds.",
        ))
    }
}

fn check_receipts(receipts: &[contracts::ActionReceipt]) -> Result<(), ApiError> {
    if receipts.len() <= 4
        && receipts.iter().all(|receipt| {
            !receipt.observation_id.is_empty()
                && receipt.observation_id.len() <= 64
                && receipt
                    .error_code
                    .as_ref()
                    .is_none_or(|error| error.len() <= 64)
                && receipt
                    .evidence
                    .resolved_role_category
                    .as_ref()
                    .is_none_or(|role| role.len() <= 128)
        })
    {
        Ok(())
    } else {
        Err(ApiError::invalid("Action receipts exceed safe bounds."))
    }
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

fn record_provider_metadata(
    provider_kind: &str,
    model: &str,
    latency: std::time::Duration,
    status: &PlannerStatus,
) {
    tracing::info!(
        component = "computer_provider",
        operation = "turn_complete",
        provider_kind,
        model,
        latency_bucket = latency_bucket(latency),
        action_kind = planner_action_kind(status),
        outcome = planner_outcome(status),
    );
}

fn latency_bucket(latency: std::time::Duration) -> &'static str {
    match latency.as_millis() {
        0..=499 => "lt_500ms",
        500..=1_999 => "500ms_2s",
        2_000..=4_999 => "2s_5s",
        5_000..=9_999 => "5s_10s",
        _ => "gte_10s",
    }
}

fn planner_outcome(status: &PlannerStatus) -> &'static str {
    match status {
        PlannerStatus::Actions { .. } => "action_proposed",
        PlannerStatus::Completed { .. } => "completed",
        PlannerStatus::NeedsUser { .. } => "needs_user",
    }
}

fn planner_action_kind(status: &PlannerStatus) -> &'static str {
    let PlannerStatus::Actions { actions } = status else {
        return "none";
    };
    actions
        .first()
        .map_or("none", |planned| match &planned.action {
            contracts::ComputerAction::ActivateApplication { .. } => "activate_application",
            contracts::ComputerAction::Element { .. } => "element",
            contracts::ComputerAction::Move { .. } => "move",
            contracts::ComputerAction::Click { .. } => "click",
            contracts::ComputerAction::Scroll { .. } => "scroll",
            contracts::ComputerAction::TypeText { .. } => "type_text",
            contracts::ComputerAction::KeyPress { .. } => "key_press",
            contracts::ComputerAction::Drag { .. } => "drag",
            contracts::ComputerAction::Wait { .. } => "wait",
            contracts::ComputerAction::Capture => "capture",
        })
}

fn limit_error(code: ErrorCode, message: &str) -> ApiError {
    ApiError {
        status: axum::http::StatusCode::TOO_MANY_REQUESTS,
        app: contracts::AppError::new(code, message, false),
    }
}
