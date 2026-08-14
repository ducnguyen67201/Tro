use std::{env, fmt, net::SocketAddr};

use thiserror::Error;
use zeroize::Zeroize;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid environment variable: {0}")]
    Invalid(&'static str),
}

pub struct SecretValue(String);

impl SecretValue {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([REDACTED])")
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub struct AppConfig {
    pub database_url: String,
    pub openai_api_key: Option<SecretValue>,
    pub openai_realtime_model: String,
    pub openai_computer_model: String,
    pub openai_realtime_voice: String,
    pub openrouter_api_key: Option<SecretValue>,
    pub openrouter_base_url: String,
    pub openrouter_model: String,
    pub openrouter_computer_model: String,
    pub tutor_timeout_seconds: u64,
    pub tutor_audio_max_bytes: usize,
    pub tutor_enabled: bool,
    pub development_device_token: Option<SecretValue>,
    pub development_invite_code: Option<SecretValue>,
    pub device_token_hmac_key: SecretValue,
    pub invite_code_pepper: SecretValue,
    pub google_oauth_client_id: Option<String>,
    pub google_oauth_client_secret: Option<SecretValue>,
    pub agent_continuation_aead_key: SecretValue,
    pub bind_addr: SocketAddr,
    pub screenshot_max_bytes: usize,
    pub screenshot_max_edge_px: u32,
    pub agent_max_turns: u32,
    pub agent_max_actions: u32,
    pub agent_max_seconds: u64,
    pub device_daily_realtime_seconds: u32,
    pub device_daily_screenshots: u32,
    pub device_daily_agent_turns: u32,
    pub agent_enabled: bool,
    pub reliable_computer_use_enabled: bool,
    pub computer_provider: ComputerProviderKind,
    pub realtime_enabled: bool,
    pub log_format: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputerProviderKind {
    OpenAiResponses,
    OpenRouterChat,
}

impl std::str::FromStr for ComputerProviderKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openai_responses" => Ok(Self::OpenAiResponses),
            "openrouter_chat" => Ok(Self::OpenRouterChat),
            _ => Err(()),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Self {
            database_url: required("DATABASE_URL")?,
            openai_api_key: optional_secret("OPENAI_API_KEY"),
            openai_realtime_model: optional("OPENAI_REALTIME_MODEL", "gpt-realtime"),
            openai_computer_model: optional("OPENAI_COMPUTER_MODEL", "gpt-5.6"),
            openai_realtime_voice: optional("OPENAI_REALTIME_VOICE", "marin"),
            openrouter_api_key: optional_secret("OPENROUTER_API_KEY"),
            openrouter_base_url: optional("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1"),
            openrouter_model: optional("OPENROUTER_MODEL", "google/gemini-2.5-flash"),
            openrouter_computer_model: optional(
                "OPENROUTER_COMPUTER_MODEL",
                "google/gemini-2.5-flash",
            ),
            tutor_timeout_seconds: parsed("TUTOR_TIMEOUT_SECONDS", "20")?,
            tutor_audio_max_bytes: parsed("TUTOR_AUDIO_MAX_BYTES", "3145728")?,
            tutor_enabled: parsed("TUTOR_ENABLED", "true")?,
            development_device_token: optional_secret("TRO_DEVICE_TOKEN"),
            development_invite_code: optional_secret("TRO_DEVELOPMENT_INVITE_CODE"),
            device_token_hmac_key: secret("DEVICE_TOKEN_HMAC_KEY")?,
            invite_code_pepper: secret("INVITE_CODE_PEPPER")?,
            google_oauth_client_id: optional_value("GOOGLE_OAUTH_CLIENT_ID"),
            google_oauth_client_secret: optional_secret("GOOGLE_OAUTH_CLIENT_SECRET"),
            agent_continuation_aead_key: secret("AGENT_CONTINUATION_AEAD_KEY")?,
            bind_addr: parsed("BIND_ADDR", "127.0.0.1:8080")?,
            screenshot_max_bytes: parsed("SCREENSHOT_MAX_BYTES", "6291456")?,
            screenshot_max_edge_px: parsed("SCREENSHOT_MAX_EDGE_PX", "3840")?,
            agent_max_turns: parsed("AGENT_MAX_TURNS", "20")?,
            agent_max_actions: parsed("AGENT_MAX_ACTIONS", "100")?,
            agent_max_seconds: parsed("AGENT_MAX_SECONDS", "300")?,
            device_daily_realtime_seconds: parsed("DEVICE_DAILY_REALTIME_SECONDS", "3600")?,
            device_daily_screenshots: parsed("DEVICE_DAILY_SCREENSHOTS", "200")?,
            device_daily_agent_turns: parsed("DEVICE_DAILY_AGENT_TURNS", "100")?,
            agent_enabled: parsed("AGENT_ENABLED", "false")?,
            reliable_computer_use_enabled: parsed("RELIABLE_COMPUTER_USE_ENABLED", "false")?,
            computer_provider: parsed("COMPUTER_PROVIDER", "openrouter_chat")?,
            realtime_enabled: parsed("REALTIME_ENABLED", "false")?,
            log_format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_owned()),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn test() -> Self {
        Self {
            database_url: "postgres://unused".to_owned(),
            openai_api_key: Some(SecretValue("test-provider-key".to_owned())),
            openai_realtime_model: "test-realtime".to_owned(),
            openai_computer_model: "test-computer".to_owned(),
            openai_realtime_voice: "test-voice".to_owned(),
            openrouter_api_key: Some(SecretValue("test-openrouter-provider-key".to_owned())),
            openrouter_base_url: "https://openrouter.ai/api/v1".to_owned(),
            openrouter_model: "test/provider-model".to_owned(),
            openrouter_computer_model: "test/computer-model".to_owned(),
            tutor_timeout_seconds: 20,
            tutor_audio_max_bytes: 3_145_728,
            tutor_enabled: true,
            development_device_token: None,
            development_invite_code: None,
            device_token_hmac_key: SecretValue("test-hmac-key-with-at-least-32-bytes".to_owned()),
            invite_code_pepper: SecretValue("test-invite-pepper".to_owned()),
            google_oauth_client_id: Some("test.apps.googleusercontent.com".to_owned()),
            google_oauth_client_secret: Some(SecretValue("test-google-secret".to_owned())),
            agent_continuation_aead_key: SecretValue("01234567890123456789012345678901".to_owned()),
            bind_addr: "127.0.0.1:0".parse().expect("static socket address"),
            screenshot_max_bytes: 6_291_456,
            screenshot_max_edge_px: 3840,
            agent_max_turns: 20,
            agent_max_actions: 100,
            agent_max_seconds: 300,
            device_daily_realtime_seconds: 3600,
            device_daily_screenshots: 200,
            device_daily_agent_turns: 100,
            agent_enabled: true,
            reliable_computer_use_enabled: true,
            computer_provider: ComputerProviderKind::OpenRouterChat,
            realtime_enabled: true,
            log_format: "compact".to_owned(),
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if (self.realtime_enabled
            || (self.agent_enabled
                && self.computer_provider == ComputerProviderKind::OpenAiResponses))
            && self.openai_api_key.is_none()
        {
            return Err(ConfigError::Missing("OPENAI_API_KEY"));
        }
        if (self.tutor_enabled
            || (self.agent_enabled
                && self.computer_provider == ComputerProviderKind::OpenRouterChat))
            && self.openrouter_api_key.is_none()
        {
            return Err(ConfigError::Missing("OPENROUTER_API_KEY"));
        }
        let base_url = url::Url::parse(&self.openrouter_base_url)
            .map_err(|_| ConfigError::Invalid("OPENROUTER_BASE_URL"))?;
        if base_url.scheme() != "https"
            || base_url.host_str() != Some("openrouter.ai")
            || base_url.port_or_known_default() != Some(443)
            || base_url.username() != ""
            || base_url.password().is_some()
            || base_url.query().is_some()
            || base_url.fragment().is_some()
            || base_url.path().trim_end_matches('/') != "/api/v1"
        {
            return Err(ConfigError::Invalid("OPENROUTER_BASE_URL"));
        }
        if !valid_model(&self.openrouter_model) || !valid_model(&self.openrouter_computer_model) {
            return Err(ConfigError::Invalid("OPENROUTER_MODEL"));
        }
        if !valid_openai_model(&self.openai_computer_model) {
            return Err(ConfigError::Invalid("OPENAI_COMPUTER_MODEL"));
        }
        if !(5..=60).contains(&self.tutor_timeout_seconds)
            || !(1024..=6_291_456).contains(&self.tutor_audio_max_bytes)
        {
            return Err(ConfigError::Invalid("TUTOR_LIMITS"));
        }
        if !(65_536..=12_582_912).contains(&self.screenshot_max_bytes)
            || !(320..=7_680).contains(&self.screenshot_max_edge_px)
            || !(1..=40).contains(&self.agent_max_turns)
            || !(1..=200).contains(&self.agent_max_actions)
            || !(30..=1_800).contains(&self.agent_max_seconds)
            || self.agent_max_actions < self.agent_max_turns
            || self.device_daily_realtime_seconds == 0
            || self.device_daily_screenshots == 0
            || self.device_daily_agent_turns == 0
        {
            return Err(ConfigError::Invalid("RUNTIME_LIMITS"));
        }
        if self.openrouter_api_key.as_ref().is_some_and(|key| {
            key.expose().len() < 20
                || key.expose().len() > 512
                || key.expose().contains(char::is_whitespace)
        }) || self.device_token_hmac_key.expose().len() < 32
            || self.invite_code_pepper.expose().len() < 16
        {
            return Err(ConfigError::Invalid("SECRET_LENGTHS"));
        }
        if self.agent_enabled && self.agent_continuation_aead_key.expose().len() != 32 {
            return Err(ConfigError::Invalid("AGENT_CONTINUATION_AEAD_KEY"));
        }
        match (
            &self.google_oauth_client_id,
            &self.google_oauth_client_secret,
        ) {
            (Some(client_id), client_secret)
                if (8..=512).contains(&client_id.len())
                    && client_id.ends_with(".apps.googleusercontent.com")
                    && !client_id.contains(char::is_whitespace)
                    && client_secret.as_ref().is_none_or(|secret| {
                        (6..=512).contains(&secret.expose().len())
                            && !secret.expose().contains(char::is_whitespace)
                    }) => {}
            (None, None) => {}
            _ => return Err(ConfigError::Invalid("GOOGLE_OAUTH_CREDENTIALS")),
        }
        if let Some(token) = &self.development_device_token
            && (self.database_url != "memory://"
                || !self.bind_addr.ip().is_loopback()
                || token.expose().len() < 32
                || token.expose().len() > 256
                || token.expose().contains(char::is_whitespace))
        {
            return Err(ConfigError::Invalid("TRO_DEVICE_TOKEN"));
        }
        if let Some(code) = &self.development_invite_code
            && (self.database_url != "memory://"
                || !self.bind_addr.ip().is_loopback()
                || self.development_device_token.is_none()
                || !(4..=128).contains(&code.expose().len())
                || !code
                    .expose()
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-'))
        {
            return Err(ConfigError::Invalid("TRO_DEVELOPMENT_INVITE_CODE"));
        }
        Ok(())
    }
}

fn valid_model(model: &str) -> bool {
    (3..=120).contains(&model.len())
        && !model.chars().any(char::is_whitespace)
        && model.contains('/')
}

fn valid_openai_model(model: &str) -> bool {
    (3..=120).contains(&model.len()) && !model.chars().any(char::is_whitespace)
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigError::Missing(name))
}

fn secret(name: &'static str) -> Result<SecretValue, ConfigError> {
    required(name).map(SecretValue)
}

fn optional(name: &'static str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
}

fn optional_value(name: &'static str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn optional_secret(name: &'static str) -> Option<SecretValue> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(SecretValue)
}

fn parsed<T>(name: &'static str, default: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    env::var(name)
        .unwrap_or_else(|_| default.to_owned())
        .parse()
        .map_err(|_| ConfigError::Invalid(name))
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ComputerProviderKind};

    #[test]
    fn rejects_provider_key_exfiltration_urls() {
        for base_url in [
            "http://openrouter.ai/api/v1",
            "https://user:pass@openrouter.ai/api/v1",
            "https://openrouter.ai:444/api/v1",
            "https://openrouter.ai/other",
            "https://openrouter.ai/api/v1?redirect=attacker",
        ] {
            let mut config = AppConfig::test();
            config.openrouter_base_url = base_url.to_owned();
            assert!(config.validate().is_err(), "accepted {base_url}");
        }
        assert!(AppConfig::test().validate().is_ok());
    }

    #[test]
    fn allows_google_desktop_clients_without_a_client_secret() {
        let mut config = AppConfig::test();
        config.google_oauth_client_secret = None;
        assert!(config.validate().is_ok());

        config.google_oauth_client_id = None;
        config.google_oauth_client_secret = Some(super::SecretValue("orphan-secret".to_owned()));
        assert!(config.validate().is_err());
    }

    #[test]
    fn selected_computer_provider_controls_the_required_secret() {
        let mut openai = AppConfig::test();
        openai.tutor_enabled = false;
        openai.computer_provider = ComputerProviderKind::OpenAiResponses;
        openai.openrouter_api_key = None;
        assert!(openai.validate().is_ok());
        openai.openai_api_key = None;
        assert!(openai.validate().is_err());

        let mut openrouter = AppConfig::test();
        openrouter.realtime_enabled = false;
        openrouter.computer_provider = ComputerProviderKind::OpenRouterChat;
        openrouter.openai_api_key = None;
        assert!(openrouter.validate().is_ok());
        openrouter.openrouter_api_key = None;
        assert!(openrouter.validate().is_err());
    }
}
