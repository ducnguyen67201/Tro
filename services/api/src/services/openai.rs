use async_trait::async_trait;
use contracts::{PlannedComputerAction, PlannerStatus, RealtimeSecretResponse};
use serde_json::{Value, json};
use zeroize::Zeroizing;

use crate::{
    config::AppConfig,
    error::ApiError,
    services::computer_provider::{
        ComputerProvider, ComputerProviderRequest, ComputerProviderTurn,
    },
};

#[async_trait]
pub trait Provider: Send + Sync {
    async fn create_realtime_secret(
        &self,
        locale: &str,
        safety_identifier_hash: &str,
    ) -> Result<RealtimeSecretResponse, ApiError>;
}

pub struct CloudProvider {
    client: reqwest::Client,
    openai_api_key: Option<Zeroizing<String>>,
    realtime_model: String,
    voice: String,
}

impl CloudProvider {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            openai_api_key: config
                .openai_api_key
                .as_ref()
                .map(|key| Zeroizing::new(key.expose().to_owned())),
            realtime_model: config.openai_realtime_model.clone(),
            voice: config.openai_realtime_voice.clone(),
        }
    }
}

#[async_trait]
impl Provider for CloudProvider {
    async fn create_realtime_secret(
        &self,
        locale: &str,
        safety_identifier_hash: &str,
    ) -> Result<RealtimeSecretResponse, ApiError> {
        let api_key = self
            .openai_api_key
            .as_ref()
            .ok_or_else(|| ApiError::disabled("realtime"))?;
        let response = self
            .client
            .post("https://api.openai.com/v1/realtime/client_secrets")
            .bearer_auth(api_key.as_str())
            .json(&json!({
                "expires_after": {"anchor": "created_at", "seconds": 600},
                "session": {
                    "type": "realtime",
                    "model": self.realtime_model,
                    "audio": {"output": {"voice": self.voice}},
                    "instructions": format!("Respond in {locale}. Safety identifier: {safety_identifier_hash}")
                }
            }))
            .send()
            .await
            .map_err(|_| ApiError::provider())?;
        let value: Value = response
            .error_for_status()
            .map_err(|_| ApiError::provider())?
            .json()
            .await
            .map_err(|_| ApiError::provider())?;
        let secret = value
            .pointer("/value")
            .or_else(|| value.pointer("/client_secret/value"))
            .and_then(Value::as_str)
            .ok_or_else(ApiError::provider)?;
        let expires_at = value
            .pointer("/expires_at")
            .or_else(|| value.pointer("/client_secret/expires_at"))
            .and_then(Value::as_i64)
            .ok_or_else(ApiError::provider)?;
        Ok(RealtimeSecretResponse {
            client_secret: secret.to_owned(),
            expires_at_unix: expires_at,
            model: self.realtime_model.clone(),
            voice: self.voice.clone(),
        })
    }
}

#[derive(Default)]
pub struct FakeProvider {
    pub actions: std::sync::Mutex<Vec<PlannedComputerAction>>,
    pub requests:
        std::sync::Mutex<Vec<crate::services::computer_provider::RecordedComputerRequest>>,
}

#[async_trait]
impl Provider for FakeProvider {
    async fn create_realtime_secret(
        &self,
        _locale: &str,
        _safety_identifier_hash: &str,
    ) -> Result<RealtimeSecretResponse, ApiError> {
        Ok(RealtimeSecretResponse {
            client_secret: "ephemeral-test-only".to_owned(),
            expires_at_unix: time::OffsetDateTime::now_utc().unix_timestamp() + 600,
            model: "test-realtime".to_owned(),
            voice: "test-voice".to_owned(),
        })
    }
}

#[async_trait]
impl ComputerProvider for FakeProvider {
    async fn turn(
        &self,
        request: ComputerProviderRequest<'_>,
    ) -> Result<ComputerProviderTurn, ApiError> {
        self.requests
            .lock()
            .expect("fake request mutex")
            .push(request.record());
        let actions = self.actions.lock().expect("fake action mutex").clone();
        let status = if actions.is_empty() {
            PlannerStatus::Completed {
                message_vi: "Đã hoàn thành.".to_owned(),
            }
        } else {
            PlannerStatus::Actions { actions }
        };
        Ok(ComputerProviderTurn {
            continuation: serde_json::json!({
                "provider": "fake",
                "goal_hash": blake3::hash(request.goal.as_bytes()).to_hex().to_string(),
                "turn": request.turn_number
            })
            .to_string(),
            status,
            provider_kind: "fake",
            model: "fake".to_owned(),
        })
    }
}
