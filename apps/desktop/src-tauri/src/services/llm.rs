use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use contracts::{
    ApiEnvelope, AppError, ErrorCode, ScreenFrameMeta, TutorTurnMetadata, TutorTurnResponse,
};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use zeroize::Zeroize;

use super::secrets;

pub const DEFAULT_BACKEND_URL: &str = "http://127.0.0.1:8080";
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 25;
const MAX_BACKEND_RESPONSE_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmConfig {
    pub backend_url: String,
    pub timeout_seconds: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend_url: DEFAULT_BACKEND_URL.to_owned(),
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
        }
    }
}

impl LlmConfig {
    pub fn load() -> Self {
        let mut config = Self::default();
        if let Ok(value) = std::env::var("TRO_API_BASE_URL") {
            config.backend_url = value;
        }
        if let Ok(value) = std::env::var("TRO_LLM_TIMEOUT_SECONDS")
            && let Ok(seconds) = value.parse()
        {
            config.timeout_seconds = seconds;
        }
        config.validate().unwrap_or_default()
    }

    pub fn validate(self) -> Result<Self, AppError> {
        let url = url::Url::parse(&self.backend_url)
            .map_err(|_| invalid_config("Địa chỉ máy chủ Tro không hợp lệ."))?;
        let secure_remote = url.scheme() == "https";
        let local_development =
            url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost"));
        if !secure_remote && !local_development {
            return Err(invalid_config("Máy chủ Tro từ xa phải sử dụng HTTPS."));
        }
        if url.username() != "" || url.password().is_some() || url.query().is_some() {
            return Err(invalid_config("Địa chỉ máy chủ Tro không hợp lệ."));
        }
        if !(5..=60).contains(&self.timeout_seconds) {
            return Err(invalid_config("Thời gian chờ phải từ 5 đến 60 giây."));
        }
        Ok(self)
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/tutor/turns", self.backend_url.trim_end_matches('/'))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct LlmConfigSnapshot {
    pub backend_url: String,
    pub timeout_seconds: u64,
    pub device_authenticated: bool,
}

impl LlmConfigSnapshot {
    pub fn from_config(config: &LlmConfig) -> Self {
        Self {
            backend_url: config.backend_url.clone(),
            timeout_seconds: config.timeout_seconds,
            device_authenticated: secrets::device_token_configured(),
        }
    }
}

pub struct LlmTurnInput {
    pub audio_wav: Vec<u8>,
    pub screenshot_jpeg: Vec<u8>,
    pub frame: ScreenFrameMeta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmTurnOutput {
    pub guidance: String,
    pub computer_goal: Option<String>,
}

impl Drop for LlmTurnInput {
    fn drop(&mut self) {
        self.audio_wav.zeroize();
        self.screenshot_jpeg.zeroize();
    }
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(
        &self,
        config: &LlmConfig,
        input: LlmTurnInput,
    ) -> Result<LlmTurnOutput, AppError>;
}

pub struct LlmGateway {
    provider: Arc<dyn LlmProvider>,
}

impl Default for LlmGateway {
    fn default() -> Self {
        Self {
            provider: Arc::new(TroBackendProvider::default()),
        }
    }
}

impl LlmGateway {
    #[cfg(test)]
    fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }

    pub async fn complete(
        &self,
        config: &LlmConfig,
        input: LlmTurnInput,
    ) -> Result<LlmTurnOutput, AppError> {
        tokio::time::timeout(
            Duration::from_secs(config.timeout_seconds),
            self.provider.complete(config, input),
        )
        .await
        .map_err(|_| {
            AppError::new(
                ErrorCode::AgentTimeout,
                format!(
                    "Tro chưa nhận được phản hồi sau {} giây. Hãy thử lại.",
                    config.timeout_seconds
                ),
                true,
            )
        })?
    }
}

pub struct TroBackendProvider {
    client: reqwest::Client,
}

impl Default for TroBackendProvider {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for TroBackendProvider {
    async fn complete(
        &self,
        config: &LlmConfig,
        mut input: LlmTurnInput,
    ) -> Result<LlmTurnOutput, AppError> {
        let token = secrets::load_device_token()?.ok_or_else(|| {
            AppError::new(
                ErrorCode::AuthExpired,
                "Tro chưa có phiên thiết bị. Hãy đăng nhập lại.",
                false,
            )
        })?;
        let metadata = serde_json::to_vec(&TutorTurnMetadata {
            locale: "vi-VN".to_owned(),
            frame: input.frame.clone(),
        })
        .map_err(|_| protocol_error())?;
        let form = Form::new()
            .part(
                "metadata",
                Part::bytes(metadata)
                    .mime_str("application/json")
                    .map_err(|_| protocol_error())?,
            )
            .part(
                "audio",
                Part::bytes(std::mem::take(&mut input.audio_wav))
                    .file_name("question.wav")
                    .mime_str("audio/wav")
                    .map_err(|_| protocol_error())?,
            )
            .part(
                "screenshot",
                Part::bytes(std::mem::take(&mut input.screenshot_jpeg))
                    .file_name("screen.jpg")
                    .mime_str("image/jpeg")
                    .map_err(|_| protocol_error())?,
            );
        let response = self
            .client
            .post(config.endpoint())
            .bearer_auth(token.as_str())
            .multipart(form)
            .send()
            .await
            .map_err(backend_unavailable)?;
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
        let envelope: ApiEnvelope<TutorTurnResponse> =
            serde_json::from_slice(&bytes).map_err(|_| protocol_error())?;
        let guidance = envelope.data.guidance.trim();
        if guidance.is_empty() || guidance.len() > 12_000 {
            return Err(protocol_error());
        }
        let computer_goal = envelope
            .data
            .computer_goal
            .map(|goal| goal.trim().to_owned())
            .filter(|goal| (3..=500).contains(&goal.len()));
        Ok(LlmTurnOutput {
            guidance: guidance.to_owned(),
            computer_goal,
        })
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

fn invalid_config(message: &str) -> AppError {
    AppError::new(ErrorCode::InvalidRequest, message, false)
}

fn backend_unavailable(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "llm",
        operation = "tro_backend_request",
        error_code = "provider_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::ProviderUnavailable,
        "Tro chưa kết nối được với máy chủ AI. Hãy kiểm tra mạng và thử lại.",
        true,
    )
}

fn protocol_error() -> AppError {
    AppError::new(
        ErrorCode::ProviderProtocolError,
        "Máy chủ Tro trả về phản hồi chưa hợp lệ. Hãy thử lại.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use async_trait::async_trait;
    use contracts::{AppError, ErrorCode, ImageMime, ScreenFrameMeta};

    use super::{LlmConfig, LlmGateway, LlmProvider, LlmTurnInput, LlmTurnOutput};

    struct SlowProvider;

    #[async_trait]
    impl LlmProvider for SlowProvider {
        async fn complete(
            &self,
            _config: &LlmConfig,
            _input: LlmTurnInput,
        ) -> Result<LlmTurnOutput, AppError> {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(LlmTurnOutput {
                guidance: "late".to_owned(),
                computer_goal: None,
            })
        }
    }

    fn input() -> LlmTurnInput {
        LlmTurnInput {
            audio_wav: Vec::new(),
            screenshot_jpeg: Vec::new(),
            frame: ScreenFrameMeta {
                frame_id: "fixture".to_owned(),
                monitor_id: "main".to_owned(),
                width_px: 1,
                height_px: 1,
                origin_x_px: 0,
                origin_y_px: 0,
                scale_factor: 1.0,
                layout_generation: 0,
                mime_type: ImageMime::Jpeg,
            },
        }
    }

    #[test]
    fn rejects_insecure_remote_or_credential_bearing_backend_urls() {
        for backend_url in [
            "http://example.com",
            "https://user:pass@example.com",
            "https://example.com?token=secret",
        ] {
            assert!(
                LlmConfig {
                    backend_url: backend_url.to_owned(),
                    timeout_seconds: 20,
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            LlmConfig {
                backend_url: "http://127.0.0.1:8080".to_owned(),
                timeout_seconds: 20,
            }
            .validate()
            .is_ok()
        );
    }

    #[tokio::test]
    async fn stops_waiting_when_the_backend_deadline_expires() {
        let gateway = LlmGateway::new(Arc::new(SlowProvider));
        let config = LlmConfig {
            timeout_seconds: 0,
            ..LlmConfig::default()
        };
        let error = gateway
            .complete(&config, input())
            .await
            .expect_err("slow provider should hit the gateway deadline");
        assert_eq!(error.code, ErrorCode::AgentTimeout);
    }
}
