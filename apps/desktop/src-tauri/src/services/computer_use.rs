use std::time::Duration;

use contracts::{
    ActionReceipt, AgentTurnMetadata, AgentTurnResponse, ApiEnvelope, AppError, ApplicationRef,
    CreateAgentRunMetadata, ErrorCode, ImageMime,
};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use crate::services::observation::Observation;

use super::{llm::LlmConfig, secrets};

const MAX_BACKEND_RESPONSE_BYTES: usize = 1_048_576;

pub struct ComputerUseGateway {
    client: reqwest::Client,
}

#[async_trait::async_trait]
pub trait ComputerUseBackend: Send + Sync {
    async fn create_run(
        &self,
        config: &LlmConfig,
        goal: &str,
        available_apps: Vec<ApplicationRef>,
        observation: &Observation,
    ) -> Result<AgentTurnResponse, AppError>;

    async fn next_turn(
        &self,
        config: &LlmConfig,
        goal: &str,
        run_id: &str,
        turn_number: u32,
        receipts: Vec<ActionReceipt>,
        observation: &Observation,
    ) -> Result<AgentTurnResponse, AppError>;

    async fn stop_run(&self, config: &LlmConfig, run_id: &str);
}

impl Default for ComputerUseGateway {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl ComputerUseBackend for ComputerUseGateway {
    async fn create_run(
        &self,
        config: &LlmConfig,
        goal: &str,
        available_apps: Vec<ApplicationRef>,
        observation: &Observation,
    ) -> Result<AgentTurnResponse, AppError> {
        let frame = observation.frame.as_ref().ok_or_else(protocol_error)?;
        let metadata = CreateAgentRunMetadata {
            goal: goal.to_owned(),
            frame: frame.meta.clone(),
            observation: observation.metadata.clone(),
            available_apps,
        };
        self.send_turn(
            config,
            &format!("{}/v1/agent/runs", config.backend_url.trim_end_matches('/')),
            None,
            serde_json::to_vec(&metadata).map_err(|_| protocol_error())?,
            observation.serialized_metadata()?.to_vec(),
            frame.bytes.clone(),
            frame.meta.mime_type,
        )
        .await
    }

    async fn next_turn(
        &self,
        config: &LlmConfig,
        goal: &str,
        run_id: &str,
        turn_number: u32,
        receipts: Vec<ActionReceipt>,
        observation: &Observation,
    ) -> Result<AgentTurnResponse, AppError> {
        let frame = observation.frame.as_ref().ok_or_else(protocol_error)?;
        let metadata = AgentTurnMetadata {
            goal: goal.to_owned(),
            turn_number,
            frame: frame.meta.clone(),
            observation: observation.metadata.clone(),
            receipts,
        };
        self.send_turn(
            config,
            &format!(
                "{}/v1/agent/runs/{run_id}/turns",
                config.backend_url.trim_end_matches('/')
            ),
            Some(uuid::Uuid::new_v4().to_string()),
            serde_json::to_vec(&metadata).map_err(|_| protocol_error())?,
            observation.serialized_metadata()?.to_vec(),
            frame.bytes.clone(),
            frame.meta.mime_type,
        )
        .await
    }

    async fn stop_run(&self, config: &LlmConfig, run_id: &str) {
        let Ok(Some(token)) = secrets::load_device_token() else {
            return;
        };
        let _response = self
            .client
            .post(format!(
                "{}/v1/agent/runs/{run_id}/stop",
                config.backend_url.trim_end_matches('/')
            ))
            .bearer_auth(token.as_str())
            .timeout(Duration::from_secs(3))
            .send()
            .await;
    }
}

impl ComputerUseGateway {
    async fn send_turn(
        &self,
        config: &LlmConfig,
        endpoint: &str,
        idempotency_key: Option<String>,
        metadata: Vec<u8>,
        observation: Vec<u8>,
        screenshot: Vec<u8>,
        screenshot_mime: ImageMime,
    ) -> Result<AgentTurnResponse, AppError> {
        let token = secrets::load_device_token()?.ok_or_else(|| {
            AppError::new(
                ErrorCode::AuthExpired,
                "Tro chưa có phiên thiết bị. Hãy đăng nhập lại.",
                false,
            )
        })?;
        let form = Form::new()
            .part(
                "metadata",
                Part::bytes(metadata)
                    .mime_str("application/json")
                    .map_err(|_| protocol_error())?,
            )
            .part(
                "observation",
                Part::bytes(observation)
                    .mime_str("application/json")
                    .map_err(|_| protocol_error())?,
            )
            .part(
                "screenshot",
                Part::bytes(screenshot)
                    .file_name(match screenshot_mime {
                        ImageMime::Jpeg => "screen.jpg",
                        ImageMime::Png => "screen.png",
                    })
                    .mime_str(match screenshot_mime {
                        ImageMime::Jpeg => "image/jpeg",
                        ImageMime::Png => "image/png",
                    })
                    .map_err(|_| protocol_error())?,
            );
        let mut request = self
            .client
            .post(endpoint)
            .bearer_auth(token.as_str())
            .timeout(Duration::from_secs(config.timeout_seconds))
            .multipart(form);
        if let Some(key) = idempotency_key {
            request = request.header("idempotency-key", key);
        }
        let response = request.send().await.map_err(backend_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            let remote = serde_json::from_slice::<RemoteErrorEnvelope>(&bytes).ok();
            return Err(remote.map_or_else(
                || backend_unavailable(status),
                |remote| {
                    let mut error = AppError::new(
                        remote.error.code,
                        remote.error.message_vi,
                        remote.error.retryable,
                    );
                    error.request_id = remote.error.request_id;
                    error
                },
            ));
        }
        let envelope: ApiEnvelope<AgentTurnResponse> =
            serde_json::from_slice(&bytes).map_err(|_| protocol_error())?;
        Ok(envelope.data)
    }
}

#[derive(Deserialize)]
struct RemoteErrorEnvelope {
    error: RemoteError,
}

#[derive(Deserialize)]
struct RemoteError {
    code: ErrorCode,
    message_vi: String,
    retryable: bool,
    request_id: Option<String>,
}

async fn read_bounded(mut response: reqwest::Response) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(backend_unavailable)? {
        if bytes.len().saturating_add(chunk.len()) > MAX_BACKEND_RESPONSE_BYTES {
            return Err(protocol_error());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn backend_unavailable(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "computer_use",
        operation = "tro_backend_request",
        error_code = "provider_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::ProviderUnavailable,
        "Tro chưa kết nối được computer use. Hãy thử lại.",
        true,
    )
}

fn protocol_error() -> AppError {
    AppError::new(
        ErrorCode::ProviderProtocolError,
        "Máy chủ trả về thao tác computer use chưa hợp lệ.",
        true,
    )
}
