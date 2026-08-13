use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use contracts::{ComputerAction, RealtimeSecretResponse};
use serde_json::{Value, json};

use crate::error::ApiError;

pub struct ProviderAgentTurn {
    pub continuation_id: String,
    pub actions: Vec<ComputerAction>,
    pub completed: bool,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn create_realtime_secret(
        &self,
        locale: &str,
        safety_identifier_hash: &str,
    ) -> Result<RealtimeSecretResponse, ApiError>;
    async fn agent_turn(
        &self,
        goal: &str,
        screenshot: &[u8],
        previous_response_id: Option<&str>,
    ) -> Result<ProviderAgentTurn, ApiError>;
}

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    realtime_model: String,
    computer_model: String,
    voice: String,
}

impl OpenAiProvider {
    pub fn new(
        api_key: String,
        realtime_model: String,
        computer_model: String,
        voice: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            realtime_model,
            computer_model,
            voice,
        }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    async fn create_realtime_secret(
        &self,
        locale: &str,
        safety_identifier_hash: &str,
    ) -> Result<RealtimeSecretResponse, ApiError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/realtime/client_secrets")
            .bearer_auth(&self.api_key)
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

    async fn agent_turn(
        &self,
        goal: &str,
        screenshot: &[u8],
        previous_response_id: Option<&str>,
    ) -> Result<ProviderAgentTurn, ApiError> {
        let image = STANDARD.encode(screenshot);
        let mut request = json!({
            "model": self.computer_model,
            "store": false,
            "tools": [{"type": "computer"}],
            "input": [{
                "role": "user",
                "content": [
                    {"type": "input_text", "text": goal},
                    {"type": "input_image", "image_url": format!("data:image/jpeg;base64,{image}")}
                ]
            }]
        });
        if let Some(previous) = previous_response_id {
            request["previous_response_id"] = Value::String(previous.to_owned());
        }
        let response = self
            .client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|_| ApiError::provider())?;
        let value: Value = response
            .error_for_status()
            .map_err(|_| ApiError::provider())?
            .json()
            .await
            .map_err(|_| ApiError::provider())?;
        normalize_agent_response(&value)
    }
}

fn normalize_agent_response(value: &Value) -> Result<ProviderAgentTurn, ApiError> {
    let continuation_id = value
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(ApiError::provider)?
        .to_owned();
    let mut actions = Vec::new();
    if let Some(output) = value.get("output").and_then(Value::as_array) {
        for item in output {
            let Some(action) = item.get("action") else {
                continue;
            };
            let canonical = serde_json::from_value::<ComputerAction>(action.clone())
                .map_err(|_| ApiError::invalid("Provider returned an unsupported action."))?;
            actions.push(canonical);
        }
    }
    Ok(ProviderAgentTurn {
        continuation_id,
        completed: actions.is_empty(),
        actions,
    })
}

#[derive(Default)]
pub struct FakeProvider {
    pub actions: std::sync::Mutex<Vec<ComputerAction>>,
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

    async fn agent_turn(
        &self,
        _goal: &str,
        _screenshot: &[u8],
        _previous_response_id: Option<&str>,
    ) -> Result<ProviderAgentTurn, ApiError> {
        Ok(ProviderAgentTurn {
            continuation_id: "response-test-only".to_owned(),
            actions: self.actions.lock().expect("fake provider mutex").clone(),
            completed: false,
        })
    }
}
