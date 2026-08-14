pub mod action;
pub mod api;
pub mod assistant;
pub mod error;
pub mod overlay;

pub use action::{
    ActionLocator, ActionOutcome, ActionReceipt, ActionReceiptEvidence, ActionTarget,
    ApplicationRef, CaptureScope, ComputerAction, ElementOperationKind, ForegroundContext, KeyCode,
    MouseButton, NormalizedPoint, NormalizedRect, ObservationBinding, PlannedComputerAction,
    PolicyDecision, PolicyReason, RiskTier, SecretText, UiElementSnapshot, UiObservationMetadata,
    UiState,
};
pub use api::*;
pub use assistant::*;
pub use error::{AppError, ErrorCode};
pub use overlay::*;
