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
    pub openai_api_key: SecretValue,
    pub openai_realtime_model: String,
    pub openai_computer_model: String,
    pub openai_realtime_voice: String,
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
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            openai_api_key: secret("OPENAI_API_KEY")?,
            openai_realtime_model: required("OPENAI_REALTIME_MODEL")?,
            openai_computer_model: required("OPENAI_COMPUTER_MODEL")?,
            openai_realtime_voice: required("OPENAI_REALTIME_VOICE")?,
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
            agent_enabled: parsed("AGENT_ENABLED", "true")?,
            realtime_enabled: parsed("REALTIME_ENABLED", "true")?,
            log_format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_owned()),
        })
    }

    pub fn test() -> Self {
        Self {
            database_url: "postgres://unused".to_owned(),
            openai_api_key: SecretValue("test-provider-key".to_owned()),
            openai_realtime_model: "test-realtime".to_owned(),
            openai_computer_model: "test-computer".to_owned(),
            openai_realtime_voice: "test-voice".to_owned(),
            device_token_hmac_key: SecretValue("test-hmac-key-with-at-least-32-bytes".to_owned()),
            invite_code_pepper: SecretValue("test-pepper".to_owned()),
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

fn parsed<T>(name: &'static str, default: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    env::var(name)
        .unwrap_or_else(|_| default.to_owned())
        .parse()
        .map_err(|_| ConfigError::Invalid(name))
}
