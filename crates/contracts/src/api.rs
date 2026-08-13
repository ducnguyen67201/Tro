use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ActionReceipt, ComputerAction, ScreenFrameMeta};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApiEnvelope<T> {
    pub data: T,
    pub request_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegisterDeviceRequest {
    pub invite_code: String,
    pub public_id: String,
    pub app_version: String,
    pub platform: String,
    pub accepted_age_scope: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeviceTokenResponse {
    pub device_token: String,
    pub expires_at_unix: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RealtimeSecretRequest {
    pub locale: String,
    pub mode: RealtimeMode,
    pub safety_identifier_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeMode {
    Tutor,
    Dictation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RealtimeSecretResponse {
    pub client_secret: String,
    pub expires_at_unix: i64,
    pub model: String,
    pub voice: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TutorTurnMetadata {
    pub locale: String,
    pub frame: ScreenFrameMeta,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TutorTurnResponse {
    pub guidance: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CreateAgentRunMetadata {
    pub goal: String,
    pub frame: ScreenFrameMeta,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AgentTurnMetadata {
    pub turn_number: u32,
    pub frame: ScreenFrameMeta,
    pub receipts: Vec<ActionReceipt>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AgentTurnResponse {
    pub run_id: String,
    pub turn_number: u32,
    pub actions: Vec<ComputerAction>,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TelemetryEvent {
    pub name: String,
    pub occurred_at_unix: i64,
    pub attributes: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TelemetryBatch {
    pub events: Vec<TelemetryEvent>,
}
