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
    pub openrouter_api_key: SecretValue,
    pub openrouter_base_url: String,
    pub openrouter_model: String,
    pub tutor_timeout_seconds: u64,
    pub tutor_audio_max_bytes: usize,
    pub tutor_enabled: bool,
    pub development_device_token: Option<SecretValue>,
    pub device_token_hmac_key: SecretValue,
    pub invite_code_pepper: SecretValue,
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
    pub realtime_enabled: bool,
    pub log_format: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Self {
            database_url: required("DATABASE_URL")?,
            openai_api_key: optional_secret("OPENAI_API_KEY"),
            openai_realtime_model: optional("OPENAI_REALTIME_MODEL", "gpt-realtime"),
            openai_computer_model: optional("OPENAI_COMPUTER_MODEL", "gpt-5"),
            openai_realtime_voice: optional("OPENAI_REALTIME_VOICE", "marin"),
            openrouter_api_key: secret("OPENROUTER_API_KEY")?,
            openrouter_base_url: optional("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1"),
            openrouter_model: optional("OPENROUTER_MODEL", "google/gemini-2.5-flash"),
            tutor_timeout_seconds: parsed("TUTOR_TIMEOUT_SECONDS", "20")?,
            tutor_audio_max_bytes: parsed("TUTOR_AUDIO_MAX_BYTES", "3145728")?,
            tutor_enabled: parsed("TUTOR_ENABLED", "true")?,
            development_device_token: optional_secret("TRO_DEVICE_TOKEN"),
            device_token_hmac_key: secret("DEVICE_TOKEN_HMAC_KEY")?,
            invite_code_pepper: secret("INVITE_CODE_PEPPER")?,
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
            openrouter_api_key: SecretValue("test-openrouter-provider-key".to_owned()),
            openrouter_base_url: "https://openrouter.ai/api/v1".to_owned(),
            openrouter_model: "test/provider-model".to_owned(),
            tutor_timeout_seconds: 20,
            tutor_audio_max_bytes: 3_145_728,
            tutor_enabled: true,
            development_device_token: None,
            device_token_hmac_key: SecretValue("test-hmac-key-with-at-least-32-bytes".to_owned()),
            invite_code_pepper: SecretValue("test-invite-pepper".to_owned()),
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
            realtime_enabled: true,
            log_format: "compact".to_owned(),
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if (self.agent_enabled || self.realtime_enabled) && self.openai_api_key.is_none() {
            return Err(ConfigError::Missing("OPENAI_API_KEY"));
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
        if self.openrouter_model.len() < 3
            || self.openrouter_model.len() > 120
            || self.openrouter_model.chars().any(char::is_whitespace)
            || !self.openrouter_model.contains('/')
        {
            return Err(ConfigError::Invalid("OPENROUTER_MODEL"));
        }
        if !(5..=60).contains(&self.tutor_timeout_seconds)
            || !(1024..=6_291_456).contains(&self.tutor_audio_max_bytes)
        {
            return Err(ConfigError::Invalid("TUTOR_LIMITS"));
        }
        if self.openrouter_api_key.expose().len() < 20
            || self.openrouter_api_key.expose().len() > 512
            || self
                .openrouter_api_key
                .expose()
                .contains(char::is_whitespace)
            || self.device_token_hmac_key.expose().len() < 32
            || self.invite_code_pepper.expose().len() < 16
        {
            return Err(ConfigError::Invalid("SECRET_LENGTHS"));
        }
        if self.agent_enabled && self.agent_continuation_aead_key.expose().len() != 32 {
            return Err(ConfigError::Invalid("AGENT_CONTINUATION_AEAD_KEY"));
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
        Ok(())
    }
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
    use super::AppConfig;

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
}
