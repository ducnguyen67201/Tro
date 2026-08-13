use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de};
use zeroize::Zeroize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema)]
pub struct NormalizedPoint {
    pub x: f32,
    pub y: f32,
}

impl NormalizedPoint {
    pub fn new(x: f32, y: f32) -> Result<Self, &'static str> {
        if is_normalized(x) && is_normalized(y) {
            Ok(Self { x, y })
        } else {
            Err("coordinates must be finite and between 0 and 1")
        }
    }
}

impl<'de> Deserialize<'de> for NormalizedPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Point {
            x: f32,
            y: f32,
        }

        let point = Point::deserialize(deserializer)?;
        Self::new(point.x, point.y).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, JsonSchema)]
pub struct NormalizedRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl NormalizedRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Result<Self, &'static str> {
        let valid = [x, y, width, height].into_iter().all(is_normalized)
            && x + width <= 1.0
            && y + height <= 1.0;
        valid
            .then_some(Self {
                x,
                y,
                width,
                height,
            })
            .ok_or("rectangle must be finite, normalized, and contained by the monitor")
    }
}

impl<'de> Deserialize<'de> for NormalizedRect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Rect {
            x: f32,
            y: f32,
            width: f32,
            height: f32,
        }

        let rect = Rect::deserialize(deserializer)?;
        Self::new(rect.x, rect.y, rect.width, rect.height).map_err(de::Error::custom)
    }
}

const fn is_normalized(value: f32) -> bool {
    value.is_finite() && value >= 0.0 && value <= 1.0
}

#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct SecretText(String);

impl SecretText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Clone for SecretText {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PartialEq for SecretText {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for SecretText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretText([REDACTED])")
    }
}

impl Drop for SecretText {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KeyCode {
    Enter,
    Escape,
    Tab,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Control,
    Alt,
    Shift,
    Meta,
    Character(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ComputerAction {
    Move {
        point: NormalizedPoint,
    },
    Click {
        point: NormalizedPoint,
        button: MouseButton,
        count: u8,
    },
    Scroll {
        delta_x: i32,
        delta_y: i32,
    },
    TypeText {
        text: SecretText,
    },
    KeyPress {
        keys: Vec<KeyCode>,
    },
    Drag {
        from: NormalizedPoint,
        to: NormalizedPoint,
    },
    Wait {
        milliseconds: u32,
    },
    Capture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionTarget {
    Benign,
    KnownEditor,
    UnknownField,
    Submit,
    Upload,
    Delete,
    Download,
    Settings,
    ExternalNavigation,
    PersonalData,
    Password,
    Otp,
    Payment,
    Banking,
    Legal,
    Medical,
    Government,
    ProctoredAssessment,
    PermissionOrSecurity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PlannedComputerAction {
    pub action: ComputerAction,
    pub target: ActionTarget,
    pub description_vi: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Low,
    Confirm,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReason {
    BenignNavigation,
    KnownEditor,
    ConsequentialAction,
    UnknownField,
    SensitiveField,
    Credentials,
    Payment,
    ProctoredAssessment,
    PrivilegeChange,
    SafeguardChange,
    GoalMismatch,
    UnsupportedAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicyDecision {
    pub tier: RiskTier,
    pub reason_code: PolicyReason,
    pub display_vi: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ForegroundContext {
    pub process_hash: String,
    pub window_generation: u64,
    pub control_role: Option<String>,
    pub is_secure: bool,
    pub is_elevated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ActionReceipt {
    pub action_index: u32,
    pub outcome: ActionOutcome,
    pub error_code: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Executed,
    Cancelled,
    Failed,
    Blocked,
}

#[cfg(test)]
mod tests {
    use super::{NormalizedPoint, NormalizedRect, SecretText};

    #[test]
    fn rejects_invalid_coordinates() {
        assert!(NormalizedPoint::new(f32::NAN, 0.2).is_err());
        assert!(NormalizedPoint::new(-0.1, 0.2).is_err());
        assert!(NormalizedRect::new(0.8, 0.1, 0.3, 0.2).is_err());
    }

    #[test]
    fn redacts_secret_debug_output() {
        let text = SecretText::new("mật khẩu giả");
        assert_eq!(format!("{text:?}"), "SecretText([REDACTED])");
    }
}
