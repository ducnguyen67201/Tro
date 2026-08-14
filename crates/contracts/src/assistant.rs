use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ApplicationRef,
    error::{AppError, ErrorCode},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssistantState {
    #[default]
    Idle,
    Capturing,
    Listening,
    Thinking,
    Speaking,
    Guiding,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssistantEvent {
    Start,
    Captured,
    Heard,
    ResponseAudio,
    Guidance,
    Complete,
    Stop,
    Fail,
}

impl AssistantState {
    pub fn transition(self, event: AssistantEvent) -> Result<Self, AppError> {
        let next = match (self, event) {
            (Self::Idle, AssistantEvent::Start) => Self::Capturing,
            (Self::Capturing, AssistantEvent::Captured) => Self::Listening,
            (Self::Capturing | Self::Listening, AssistantEvent::Heard) => Self::Thinking,
            (Self::Thinking, AssistantEvent::ResponseAudio) => Self::Speaking,
            (Self::Thinking | Self::Speaking, AssistantEvent::Guidance) => Self::Guiding,
            (Self::Speaking | Self::Guiding, AssistantEvent::Complete)
            | (_, AssistantEvent::Stop) => Self::Idle,
            (_, AssistantEvent::Fail) => Self::Failed,
            (Self::Failed, AssistantEvent::Complete) => Self::Idle,
            _ => {
                return Err(AppError::new(
                    ErrorCode::InvalidTransition,
                    "Trạng thái trợ lý không hợp lệ.",
                    false,
                ));
            }
        };
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CursorCompanionPhase {
    #[default]
    Hidden,
    Following,
    Acting,
    Anchored,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CursorCompanionSnapshot {
    pub phase: CursorCompanionPhase,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    #[default]
    Idle,
    ResolvingApp,
    AwaitingAppApproval,
    ActivatingApp,
    Planning,
    Validating,
    AwaitingConfirmation,
    Executing,
    Stabilizing,
    Observing,
    StaleRecovery,
    NeedsUser,
    PausedByUser,
    Completed,
    Stopped,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentEvent {
    Start,
    ActionsReady,
    ConfirmationRequired,
    Confirm,
    Executed,
    Observed,
    Complete,
    Stop,
    Fail,
}

impl AgentState {
    pub fn transition(self, event: AgentEvent) -> Result<Self, AppError> {
        let next = match (self, event) {
            (Self::Idle, AgentEvent::Start) => Self::ResolvingApp,
            (Self::ResolvingApp, AgentEvent::Observed) => Self::Planning,
            (Self::Planning, AgentEvent::ActionsReady)
            | (Self::AwaitingConfirmation, AgentEvent::Confirm) => Self::Executing,
            (Self::Planning, AgentEvent::ConfirmationRequired) => Self::AwaitingConfirmation,
            (Self::Executing, AgentEvent::Executed) => Self::Stabilizing,
            (Self::Stabilizing | Self::Observing | Self::StaleRecovery, AgentEvent::Observed) => {
                Self::Planning
            }
            (Self::Planning | Self::Observing | Self::Stabilizing, AgentEvent::Complete) => {
                Self::Completed
            }
            (_, AgentEvent::Stop) => Self::Stopped,
            (_, AgentEvent::Fail) => Self::Failed,
            _ => {
                return Err(AppError::new(
                    ErrorCode::InvalidTransition,
                    "Trạng thái tác nhân không hợp lệ.",
                    false,
                ));
            }
        };
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    NotDetermined,
    Granted,
    Denied,
    Restricted,
    Unavailable,
    RestartRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PermissionSnapshot {
    pub microphone: PermissionStatus,
    pub screen_capture: PermissionStatus,
    pub input_control: PermissionStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AssistantUiState {
    pub assistant: AssistantState,
    pub agent: AgentState,
    pub transcript: Option<String>,
    pub status_vi: String,
    pub capture_active: bool,
    pub scoped_app_name: Option<String>,
    pub agent_choices: Vec<ApplicationRef>,
}

impl Default for AssistantUiState {
    fn default() -> Self {
        Self {
            assistant: AssistantState::Idle,
            agent: AgentState::Idle,
            transcript: None,
            status_vi: "Sẵn sàng".to_owned(),
            capture_active: false,
            scoped_app_name: None,
            agent_choices: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AssistantEvent, AssistantState};

    #[test]
    fn release_can_finish_while_capture_is_running() {
        assert_eq!(
            AssistantState::Capturing.transition(AssistantEvent::Heard),
            Ok(AssistantState::Thinking)
        );
    }

    #[test]
    fn release_can_finish_after_capture() {
        assert_eq!(
            AssistantState::Listening.transition(AssistantEvent::Heard),
            Ok(AssistantState::Thinking)
        );
    }

    #[test]
    fn duplicate_release_is_rejected() {
        assert!(
            AssistantState::Thinking
                .transition(AssistantEvent::Heard)
                .is_err()
        );
    }

    #[test]
    fn stop_returns_every_active_state_to_idle() {
        for state in [
            AssistantState::Capturing,
            AssistantState::Listening,
            AssistantState::Thinking,
            AssistantState::Speaking,
            AssistantState::Guiding,
            AssistantState::Failed,
        ] {
            assert_eq!(
                state.transition(AssistantEvent::Stop),
                Ok(AssistantState::Idle)
            );
        }
    }
}
