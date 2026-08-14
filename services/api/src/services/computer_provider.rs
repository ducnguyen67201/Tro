use async_trait::async_trait;
use contracts::{
    ActionLocator, ActionReceipt, ActionTarget, ApplicationRef, ComputerAction,
    ElementOperationKind, ImageMime, KeyCode, MouseButton, NormalizedPoint, PlannedComputerAction,
    PlannerStatus, SecretText, UiObservationMetadata,
};
use serde::Deserialize;

use crate::error::ApiError;

pub const MAX_PROVIDER_RESPONSE_BYTES: usize = 1_048_576;
pub const MAX_OBSERVATION_BYTES: usize = 131_072;
pub const MAX_OBSERVATION_ELEMENTS: usize = 800;

pub struct ComputerProviderRequest<'a> {
    pub goal: &'a str,
    pub turn_number: u32,
    pub observation: &'a UiObservationMetadata,
    pub available_apps: &'a [ApplicationRef],
    pub receipts: &'a [ActionReceipt],
    pub screenshot: &'a [u8],
    pub screenshot_mime: ImageMime,
    pub continuation: Option<&'a str>,
}

impl ComputerProviderRequest<'_> {
    pub fn record(&self) -> RecordedComputerRequest {
        RecordedComputerRequest {
            goal: self.goal.to_owned(),
            turn_number: self.turn_number,
            observation_id: self.observation.binding.observation_id.clone(),
            receipts: self.receipts.to_vec(),
            has_continuation: self.continuation.is_some(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecordedComputerRequest {
    pub goal: String,
    pub turn_number: u32,
    pub observation_id: String,
    pub receipts: Vec<ActionReceipt>,
    pub has_continuation: bool,
}

pub struct ComputerProviderTurn {
    pub continuation: String,
    pub status: PlannerStatus,
    pub provider_kind: &'static str,
    pub model: String,
}

#[async_trait]
pub trait ComputerProvider: Send + Sync {
    async fn turn(
        &self,
        request: ComputerProviderRequest<'_>,
    ) -> Result<ComputerProviderTurn, ApiError>;
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolArguments {
    pub kind: ToolKind,
    pub observation_id: String,
    pub locator: ToolLocator,
    pub target: ActionTarget,
    pub description_vi: String,
    pub operation: Option<ElementOperationKind>,
    pub app_id: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub to_x: Option<f32>,
    pub to_y: Option<f32>,
    pub button: Option<MouseButton>,
    pub count: Option<u8>,
    pub delta_x: Option<i32>,
    pub delta_y: Option<i32>,
    pub text: Option<String>,
    pub keys: Option<Vec<String>>,
    pub milliseconds: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolLocator {
    pub kind: ToolLocatorKind,
    pub app_id: Option<String>,
    pub element_id: Option<String>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolLocatorKind {
    Application,
    Element,
    Frame,
}

impl ToolLocator {
    fn normalize(self) -> Result<ActionLocator, ApiError> {
        match (self.kind, self.app_id, self.element_id) {
            (ToolLocatorKind::Application, Some(app_id), None) if !app_id.is_empty() => {
                Ok(ActionLocator::Application { app_id })
            }
            (ToolLocatorKind::Element, None, Some(element_id)) if !element_id.is_empty() => {
                Ok(ActionLocator::Element { element_id })
            }
            (ToolLocatorKind::Frame, None, None) => Ok(ActionLocator::Frame),
            _ => Err(action_invalid()),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    ActivateApplication,
    Element,
    Move,
    Click,
    Scroll,
    TypeText,
    KeyPress,
    Drag,
    Wait,
    Capture,
    Finish,
    NeedsUser,
}

pub fn normalize_tool_arguments(
    arguments: &str,
    request: &ComputerProviderRequest<'_>,
) -> Result<PlannerStatus, ApiError> {
    let arguments: ToolArguments = serde_json::from_str(arguments)
        .map_err(|_| ApiError::invalid("Provider returned an invalid computer action."))?;
    if arguments.observation_id != request.observation.binding.observation_id {
        return Err(ApiError::invalid("Provider used a stale observation ID."));
    }
    let description_vi = arguments.description_vi.trim().to_owned();
    if description_vi.is_empty() || description_vi.len() > 160 {
        return Err(action_invalid());
    }
    if matches!(arguments.kind, ToolKind::Finish) {
        return Ok(PlannerStatus::Completed {
            message_vi: description_vi,
        });
    }
    if matches!(arguments.kind, ToolKind::NeedsUser) {
        return Ok(PlannerStatus::NeedsUser {
            reason_code: "provider_needs_user".to_owned(),
            message_vi: description_vi,
            choices: Vec::new(),
        });
    }
    let action = planned_action(arguments, description_vi, request)?;
    Ok(PlannerStatus::Actions {
        actions: vec![action],
    })
}

fn planned_action(
    arguments: ToolArguments,
    description_vi: String,
    request: &ComputerProviderRequest<'_>,
) -> Result<PlannedComputerAction, ApiError> {
    let point = || {
        NormalizedPoint::new(
            arguments.x.ok_or_else(action_invalid)?,
            arguments.y.ok_or_else(action_invalid)?,
        )
        .map_err(|_| action_invalid())
    };
    let action = match arguments.kind {
        ToolKind::ActivateApplication => ComputerAction::ActivateApplication {
            app_id: arguments.app_id.clone().ok_or_else(action_invalid)?,
        },
        ToolKind::Element => ComputerAction::Element {
            operation: arguments.operation.ok_or_else(action_invalid)?,
            value: arguments.text.map(SecretText::new),
        },
        ToolKind::Move => ComputerAction::Move { point: point()? },
        ToolKind::Click => ComputerAction::Click {
            point: point()?,
            button: arguments.button.unwrap_or(MouseButton::Left),
            count: arguments.count.unwrap_or(1).clamp(1, 2),
        },
        ToolKind::Scroll => ComputerAction::Scroll {
            delta_x: arguments.delta_x.unwrap_or_default().clamp(-12, 12),
            delta_y: arguments.delta_y.unwrap_or_default().clamp(-12, 12),
        },
        ToolKind::TypeText => {
            let text = arguments.text.unwrap_or_default();
            if text.is_empty() || text.len() > 2_000 {
                return Err(action_invalid());
            }
            ComputerAction::TypeText {
                text: SecretText::new(text),
            }
        }
        ToolKind::KeyPress => {
            let keys = arguments.keys.ok_or_else(action_invalid)?;
            if keys.is_empty() || keys.len() > 4 {
                return Err(action_invalid());
            }
            ComputerAction::KeyPress {
                keys: keys
                    .into_iter()
                    .map(|key| parse_key(&key))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        ToolKind::Drag => ComputerAction::Drag {
            from: point()?,
            to: NormalizedPoint::new(
                arguments.to_x.ok_or_else(action_invalid)?,
                arguments.to_y.ok_or_else(action_invalid)?,
            )
            .map_err(|_| action_invalid())?,
        },
        ToolKind::Wait => ComputerAction::Wait {
            milliseconds: arguments.milliseconds.unwrap_or(500).clamp(50, 5_000),
        },
        ToolKind::Capture => ComputerAction::Capture,
        ToolKind::Finish | ToolKind::NeedsUser => return Err(action_invalid()),
    };
    let locator = arguments.locator.normalize()?;
    let planned = PlannedComputerAction {
        observation_id: request.observation.binding.observation_id.clone(),
        locator,
        action,
        target: arguments.target,
        description_vi,
    };
    validate_binding(&planned, request)?;
    Ok(planned)
}

pub fn validate_binding(
    planned: &PlannedComputerAction,
    request: &ComputerProviderRequest<'_>,
) -> Result<(), ApiError> {
    if planned.observation_id != request.observation.binding.observation_id {
        return Err(action_invalid());
    }
    match (&planned.locator, &planned.action) {
        (
            ActionLocator::Application { app_id },
            ComputerAction::ActivateApplication { app_id: action_app },
        ) if app_id == action_app
            && (request
                .available_apps
                .iter()
                .any(|app| app.app_id == *app_id)
                || request.observation.binding.app_id == *app_id) =>
        {
            Ok(())
        }
        (ActionLocator::Element { element_id }, ComputerAction::Element { operation, .. })
            if request.observation.elements.iter().any(|element| {
                element.element_id == *element_id && element.operations.contains(operation)
            }) =>
        {
            Ok(())
        }
        (
            ActionLocator::Frame,
            ComputerAction::Move { .. }
            | ComputerAction::Click { .. }
            | ComputerAction::Scroll { .. }
            | ComputerAction::TypeText { .. }
            | ComputerAction::KeyPress { .. }
            | ComputerAction::Drag { .. }
            | ComputerAction::Wait { .. }
            | ComputerAction::Capture,
        ) => Ok(()),
        _ => Err(action_invalid()),
    }
}

fn parse_key(key: &str) -> Result<KeyCode, ApiError> {
    Ok(match key {
        "enter" => KeyCode::Enter,
        "escape" => KeyCode::Escape,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "arrow_up" => KeyCode::ArrowUp,
        "arrow_down" => KeyCode::ArrowDown,
        "arrow_left" => KeyCode::ArrowLeft,
        "arrow_right" => KeyCode::ArrowRight,
        "control" => KeyCode::Control,
        "alt" => KeyCode::Alt,
        "shift" => KeyCode::Shift,
        "meta" => KeyCode::Meta,
        "space" => KeyCode::Character(" ".to_owned()),
        _ => return Err(action_invalid()),
    })
}

fn action_invalid() -> ApiError {
    ApiError::invalid("Provider returned an incomplete or mismatched computer action.")
}

pub async fn read_bounded(mut response: reqwest::Response) -> Result<Vec<u8>, ApiError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ApiError::provider())? {
        if bytes.len().saturating_add(chunk.len()) > MAX_PROVIDER_RESPONSE_BYTES {
            return Err(ApiError::provider());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
