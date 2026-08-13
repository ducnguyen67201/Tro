use std::time::Duration;

use contracts::{ApiEnvelope, AppError, DeviceTokenResponse, ErrorCode, RegisterDeviceRequest};
use reqwest::Response;
use serde::Deserialize;
use zeroize::Zeroizing;

use super::{llm::LlmConfig, secrets};

const MAX_AUTH_RESPONSE_BYTES: usize = 65_536;

pub struct AuthGateway {
    client: reqwest::Client,
}

impl Default for AuthGateway {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl AuthGateway {
    pub async fn restore_session(&self, config: &LlmConfig) -> Result<bool, AppError> {
        let Some(token) = secrets::load_device_token()? else {
            return Ok(false);
        };
        let response = self
            .client
            .post(format!(
                "{}/v1/devices/refresh",
                config.backend_url.trim_end_matches('/')
            ))
            .bearer_auth(token.as_str())
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(auth_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                secrets::delete_device_token()?;
                return Ok(false);
            }
            return Err(remote_error(&bytes).unwrap_or_else(|| auth_unavailable(status)));
        }
        save_session(&bytes)?;
        Ok(true)
    }

    pub async fn sign_in(
        &self,
        config: &LlmConfig,
        invite_code: &str,
        accepted_age_scope: bool,
    ) -> Result<(), AppError> {
        let invite_code = validate_invite(invite_code)?;
        if !accepted_age_scope {
            return Err(AppError::new(
                ErrorCode::InvalidRequest,
                "Bạn cần xác nhận đủ 18 tuổi để tiếp tục.",
                false,
            ));
        }
        let response = self
            .client
            .post(format!(
                "{}/v1/devices/register",
                config.backend_url.trim_end_matches('/')
            ))
            .timeout(Duration::from_secs(10))
            .json(&RegisterDeviceRequest {
                invite_code: invite_code.to_owned(),
                public_id: uuid::Uuid::new_v4().to_string(),
                app_version: env!("CARGO_PKG_VERSION").to_owned(),
                platform: std::env::consts::OS.to_owned(),
                accepted_age_scope,
            })
            .send()
            .await
            .map_err(auth_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            return Err(remote_error(&bytes).unwrap_or_else(|| auth_unavailable(status)));
        }
        save_session(&bytes)
    }
}

fn save_session(bytes: &[u8]) -> Result<(), AppError> {
    let envelope: ApiEnvelope<DeviceTokenResponse> =
        serde_json::from_slice(bytes).map_err(|_| auth_protocol_error())?;
    let token = Zeroizing::new(envelope.data.device_token);
    if !(32..=256).contains(&token.len()) || token.contains(char::is_whitespace) {
        return Err(auth_protocol_error());
    }
    secrets::save_device_token(token.as_str())
}

fn remote_error(bytes: &[u8]) -> Option<AppError> {
    serde_json::from_slice::<RemoteErrorEnvelope>(bytes)
        .map(|remote| remote.error)
        .ok()
}

fn validate_invite(invite: &str) -> Result<&str, AppError> {
    let invite = invite.trim();
    if !(4..=128).contains(&invite.len())
        || !invite
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(AppError::new(
            ErrorCode::InviteInvalid,
            "Mã truy cập không đúng định dạng.",
            false,
        ));
    }
    Ok(invite)
}

async fn read_bounded(mut response: Response) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(auth_unavailable)? {
        if bytes.len().saturating_add(chunk.len()) > MAX_AUTH_RESPONSE_BYTES {
            return Err(auth_protocol_error());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[derive(Deserialize)]
struct RemoteErrorEnvelope {
    error: AppError,
}

fn auth_unavailable(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "auth",
        operation = "register_device",
        error_code = "provider_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::ProviderUnavailable,
        "Tro chưa kết nối được máy chủ. Hãy thử lại.",
        true,
    )
}

fn auth_protocol_error() -> AppError {
    AppError::new(
        ErrorCode::ProviderProtocolError,
        "Máy chủ đăng nhập trả về dữ liệu chưa hợp lệ.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::validate_invite;

    #[test]
    fn invite_accepts_only_bounded_ascii_code_characters() {
        assert_eq!(validate_invite(" TRO-LOCAL ").unwrap(), "TRO-LOCAL");
        assert!(validate_invite("bad code").is_err());
        assert!(validate_invite("x").is_err());
        assert!(validate_invite("TRO-💜").is_err());
    }
}
