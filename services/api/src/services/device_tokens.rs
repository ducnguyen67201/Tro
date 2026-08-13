use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use contracts::{DeviceTokenResponse, RegisterDeviceRequest};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

type HmacSha256 = Hmac<Sha256>;

pub async fn register_device(
    state: &AppState,
    request: RegisterDeviceRequest,
) -> Result<DeviceTokenResponse, ApiError> {
    if !request.accepted_age_scope
        || request.public_id.len() > 128
        || request.app_version.len() > 32
        || request.platform.len() > 32
    {
        return Err(ApiError::invalid(
            "Tro thử nghiệm chỉ dành cho sinh viên đại học từ 18 tuổi.",
        ));
    }
    if state
        .config
        .development_invite_code
        .as_ref()
        .is_some_and(|code| code.expose() == request.invite_code)
        && let Some(token) = &state.config.development_device_token
    {
        return Ok(development_session(token.expose()));
    }
    let candidate = format!(
        "{}{}",
        request.invite_code,
        state.config.invite_code_pepper.expose()
    );
    let invite = state
        .repository
        .active_invites()
        .await
        .map_err(|_| ApiError::provider())?
        .into_iter()
        .find(|invite| {
            PasswordHash::new(&invite.code_hash).is_ok_and(|hash| {
                Argon2::default()
                    .verify_password(candidate.as_bytes(), &hash)
                    .is_ok()
            })
        })
        .ok_or_else(|| ApiError {
            status: axum::http::StatusCode::UNAUTHORIZED,
            app: contracts::AppError::new(
                contracts::ErrorCode::InviteInvalid,
                "Mã mời không hợp lệ hoặc đã hết hạn.",
                false,
            ),
        })?;

    let public_hash = hex_sha256(request.public_id.as_bytes());
    let device_id = state
        .repository
        .redeem_invite_and_create_device(
            invite.id,
            &public_hash,
            &request.app_version,
            &request.platform,
        )
        .await
        .map_err(|_| ApiError::provider())?
        .ok_or_else(|| ApiError::invalid("Mã mời vừa được sử dụng hết."))?;
    issue_token(state, device_id).await
}

pub async fn authenticate(state: &AppState, bearer: &str) -> Result<Uuid, ApiError> {
    let digest = token_digest(state.config.device_token_hmac_key.expose(), bearer)?;
    state
        .repository
        .device_for_token(&digest)
        .await
        .map_err(|_| ApiError::provider())?
        .ok_or_else(ApiError::unauthorized)
}

pub async fn rotate_token(
    state: &AppState,
    device_id: Uuid,
    old_token: &str,
) -> Result<DeviceTokenResponse, ApiError> {
    if state
        .config
        .development_device_token
        .as_ref()
        .is_some_and(|token| token.expose() == old_token)
    {
        return Ok(development_session(old_token));
    }
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token = URL_SAFE_NO_PAD.encode(bytes);
    let old_digest = token_digest(state.config.device_token_hmac_key.expose(), old_token)?;
    let new_digest = token_digest(state.config.device_token_hmac_key.expose(), &token)?;
    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::days(30);
    state
        .repository
        .rotate_device_token(
            &old_digest,
            device_id,
            &new_digest,
            expires_at,
            now + Duration::seconds(60),
        )
        .await
        .map_err(|_| ApiError::provider())?;
    Ok(DeviceTokenResponse {
        device_token: token,
        expires_at_unix: expires_at.unix_timestamp(),
    })
}

fn development_session(token: &str) -> DeviceTokenResponse {
    DeviceTokenResponse {
        device_token: token.to_owned(),
        expires_at_unix: (OffsetDateTime::now_utc() + Duration::days(30)).unix_timestamp(),
    }
}

pub fn bearer_value(header: Option<&str>) -> Result<&str, ApiError> {
    header
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.len() >= 32 && value.len() <= 256)
        .ok_or_else(ApiError::unauthorized)
}

async fn issue_token(state: &AppState, device_id: Uuid) -> Result<DeviceTokenResponse, ApiError> {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token = URL_SAFE_NO_PAD.encode(bytes);
    let digest = token_digest(state.config.device_token_hmac_key.expose(), &token)?;
    let expires_at = OffsetDateTime::now_utc() + Duration::days(30);
    state
        .repository
        .store_device_token(device_id, &digest, expires_at)
        .await
        .map_err(|_| ApiError::provider())?;
    Ok(DeviceTokenResponse {
        device_token: token,
        expires_at_unix: expires_at.unix_timestamp(),
    })
}

pub fn token_digest(key: &str, token: &str) -> Result<String, ApiError> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| ApiError::provider())?;
    mac.update(token.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn hex_sha256(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
