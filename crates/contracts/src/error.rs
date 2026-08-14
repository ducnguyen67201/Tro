use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    InvalidTransition,
    InviteInvalid,
    AuthExpired,
    RateLimited,
    MicrophoneUnavailable,
    MicrophonePermissionDenied,
    ScreenPermissionDenied,
    AccessibilityPermissionDenied,
    ShortcutConflict,
    CaptureFailed,
    ProviderUnavailable,
    ProviderProtocolError,
    StaleObservation,
    TargetAppUnavailable,
    AmbiguousApp,
    UserTakeover,
    UnsupportedAction,
    ActionRequiresConfirmation,
    ActionBlocked,
    ElevatedTargetUnsupported,
    AgentTurnLimit,
    AgentTimeout,
    Cancelled,
    Internal,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidTransition => "invalid_transition",
            Self::InviteInvalid => "invite_invalid",
            Self::AuthExpired => "auth_expired",
            Self::RateLimited => "rate_limited",
            Self::MicrophoneUnavailable => "microphone_unavailable",
            Self::MicrophonePermissionDenied => "microphone_permission_denied",
            Self::ScreenPermissionDenied => "screen_permission_denied",
            Self::AccessibilityPermissionDenied => "accessibility_permission_denied",
            Self::ShortcutConflict => "shortcut_conflict",
            Self::CaptureFailed => "capture_failed",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::ProviderProtocolError => "provider_protocol_error",
            Self::StaleObservation => "stale_observation",
            Self::TargetAppUnavailable => "target_app_unavailable",
            Self::AmbiguousApp => "ambiguous_app",
            Self::UserTakeover => "user_takeover",
            Self::UnsupportedAction => "unsupported_action",
            Self::ActionRequiresConfirmation => "action_requires_confirmation",
            Self::ActionBlocked => "action_blocked",
            Self::ElevatedTargetUnsupported => "elevated_target_unsupported",
            Self::AgentTurnLimit => "agent_turn_limit",
            Self::AgentTimeout => "agent_timeout",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AppError {
    pub code: ErrorCode,
    pub message_vi: String,
    pub retryable: bool,
    pub request_id: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, message_vi: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message_vi: message_vi.into(),
            retryable,
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl std::error::Error for AppError {}
