use std::sync::Arc;

use contracts::{
    ActionLocator, AppError, ApplicationRef, ComputerAction, ElementOperationKind, ErrorCode,
    NormalizedPoint, PlannedComputerAction, UiState,
};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::observation::ResolvedElement,
    services::{
        application::ApplicationBackend,
        input::InputBackend,
        observation::{Observation, ObservationBackend, ObservationMode},
        user_activity::UserActivityBackend,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedActionEvidence {
    pub app_match: bool,
    pub window_match: bool,
    pub layout_match: bool,
    pub secure: bool,
    pub elevated: bool,
    pub system_ui: bool,
    pub role_category: Option<String>,
    pub supported_operation: bool,
    pub editable: bool,
    pub visual_fallback: bool,
    pub local_destructive: bool,
}

pub trait ActionExecutor: Send + Sync {
    fn validate(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &PlannedComputerAction,
    ) -> Result<ResolvedActionEvidence, AppError>;

    fn revalidate_after_confirmation(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &PlannedComputerAction,
    ) -> Result<ResolvedActionEvidence, AppError> {
        self.validate(app, observation, planned)
    }

    fn execute(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &PlannedComputerAction,
        cancellation: &CancellationToken,
    ) -> Result<(), AppError>;

    fn release_all(&self) -> Result<(), AppError>;
}

pub struct SemanticFirstExecutor {
    applications: Arc<dyn ApplicationBackend>,
    observer: Arc<dyn ObservationBackend>,
    input: Arc<dyn InputBackend>,
    activity: Arc<dyn UserActivityBackend>,
}

impl SemanticFirstExecutor {
    pub fn new(
        applications: Arc<dyn ApplicationBackend>,
        observer: Arc<dyn ObservationBackend>,
        input: Arc<dyn InputBackend>,
        activity: Arc<dyn UserActivityBackend>,
    ) -> Self {
        Self {
            applications,
            observer,
            input,
            activity,
        }
    }

    fn current_binding_matches(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
    ) -> Result<bool, AppError> {
        let state = self.applications.identity_state(&app.app_id)?;
        if !state.focused || !state.visible {
            return Err(user_takeover());
        }
        let current = self.observer.observe(app, ObservationMode::Lightweight)?;
        let expected = &observation.metadata.binding;
        let actual = &current.metadata.binding;
        Ok(expected.app_id == actual.app_id
            && expected.window_generation == actual.window_generation
            && expected.layout_generation == actual.layout_generation)
    }

    fn restore_after_confirmation(&self, app: &ApplicationRef) -> Result<(), AppError> {
        self.applications.restore_window(&app.app_id)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(750);
        loop {
            if self
                .applications
                .identity_state(&app.app_id)
                .is_ok_and(|state| state.focused && state.visible)
            {
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(stale());
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
    }

    fn element<'a>(
        observation: &'a Observation,
        planned: &PlannedComputerAction,
    ) -> Result<&'a ResolvedElement, AppError> {
        let ActionLocator::Element { element_id } = &planned.locator else {
            return Err(unsupported_binding());
        };
        observation
            .registry
            .resolve(&planned.observation_id, element_id)
    }

    fn execute_element_fallback(
        &self,
        observation: &Observation,
        element: &ResolvedElement,
        operation: ElementOperationKind,
        value: Option<&contracts::SecretText>,
        cancellation: &CancellationToken,
    ) -> Result<(), AppError> {
        let frame = observation.frame.as_ref().ok_or_else(unsupported_binding)?;
        let bounds = element.bounds.ok_or_else(unsupported_binding)?;
        let center = NormalizedPoint::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        )
        .map_err(|_| unsupported_binding())?;
        let click = ComputerAction::Click {
            point: center,
            button: contracts::MouseButton::Left,
            count: 1,
        };
        self.input.execute(&click, &frame.meta, cancellation)?;
        if operation == ElementOperationKind::SetValue {
            let value = value.ok_or_else(unsupported_binding)?;
            self.input.execute(
                &ComputerAction::TypeText {
                    text: value.clone(),
                },
                &frame.meta,
                cancellation,
            )?;
        }
        Ok(())
    }
}

impl ActionExecutor for SemanticFirstExecutor {
    fn validate(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &PlannedComputerAction,
    ) -> Result<ResolvedActionEvidence, AppError> {
        if planned.observation_id != observation.metadata.binding.observation_id {
            return Err(stale());
        }
        let current_match = self.current_binding_matches(app, observation)?;
        if !current_match {
            return Err(stale());
        }
        let mut evidence = ResolvedActionEvidence {
            app_match: app.app_id == observation.metadata.binding.app_id,
            window_match: true,
            layout_match: true,
            secure: observation.foreground.is_secure,
            elevated: observation.foreground.is_elevated,
            system_ui: false,
            role_category: observation.foreground.control_role.clone(),
            supported_operation: true,
            editable: false,
            visual_fallback: matches!(planned.locator, ActionLocator::Frame),
            local_destructive: false,
        };
        match (&planned.locator, &planned.action) {
            (
                ActionLocator::Application { app_id },
                ComputerAction::ActivateApplication { app_id: action_app },
            ) if app_id == action_app && app_id == &app.app_id => {}
            (ActionLocator::Element { .. }, ComputerAction::Element { operation, .. }) => {
                let element = Self::element(observation, planned)?;
                evidence.secure |= element.states.contains(&UiState::Secure);
                evidence.editable = element.states.contains(&UiState::Editable);
                evidence.supported_operation = element.operations.contains(operation);
                evidence.role_category = Some(element.role_category.clone());
                evidence.local_destructive =
                    element.destructive_hint || destructive_role_hint(&element.role_category);
                // Native semantic invocation is attempted by platform adapters when
                // they provide a handle. The bounded coordinate path is explicit.
                evidence.visual_fallback = element.native_token == 0;
                if !evidence.supported_operation {
                    return Err(unsupported_binding());
                }
            }
            (ActionLocator::Frame, action) if is_visual_action(action) => {}
            _ => return Err(unsupported_binding()),
        }
        if !evidence.app_match {
            return Err(stale());
        }
        Ok(evidence)
    }

    fn revalidate_after_confirmation(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &PlannedComputerAction,
    ) -> Result<ResolvedActionEvidence, AppError> {
        self.restore_after_confirmation(app)?;
        self.validate(app, observation, planned)
    }

    fn execute(
        &self,
        app: &ApplicationRef,
        observation: &Observation,
        planned: &PlannedComputerAction,
        cancellation: &CancellationToken,
    ) -> Result<(), AppError> {
        let _evidence = self.validate(app, observation, planned)?;
        if cancellation.is_cancelled() {
            return Err(cancelled());
        }
        let _lease = self.activity.begin_synthetic_input();
        match (&planned.locator, &planned.action) {
            (ActionLocator::Application { .. }, ComputerAction::ActivateApplication { .. }) => {
                self.applications.launch_or_activate(app)
            }
            (ActionLocator::Element { .. }, ComputerAction::Element { operation, value }) => {
                let element = Self::element(observation, planned)?;
                self.execute_element_fallback(
                    observation,
                    element,
                    *operation,
                    value.as_ref(),
                    cancellation,
                )
            }
            (ActionLocator::Frame, action) => {
                let frame = observation.frame.as_ref().ok_or_else(unsupported_binding)?;
                self.input.execute(action, &frame.meta, cancellation)
            }
            _ => Err(unsupported_binding()),
        }
    }

    fn release_all(&self) -> Result<(), AppError> {
        self.input.release_all()
    }
}

fn is_visual_action(action: &ComputerAction) -> bool {
    matches!(
        action,
        ComputerAction::Move { .. }
            | ComputerAction::Click { .. }
            | ComputerAction::Scroll { .. }
            | ComputerAction::TypeText { .. }
            | ComputerAction::KeyPress { .. }
            | ComputerAction::Drag { .. }
            | ComputerAction::Wait { .. }
            | ComputerAction::Capture
    )
}

fn stale() -> AppError {
    AppError::new(
        ErrorCode::StaleObservation,
        "Giao diện đã thay đổi; Tro sẽ quan sát lại.",
        true,
    )
}

fn unsupported_binding() -> AppError {
    AppError::new(
        ErrorCode::UnsupportedAction,
        "Thao tác không khớp với giao diện đã quan sát.",
        false,
    )
}

fn cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "Đã dừng computer use.", false)
}

fn user_takeover() -> AppError {
    AppError::new(
        ErrorCode::UserTakeover,
        "Ứng dụng khác đã nhận điều khiển — Tro đã tạm dừng.",
        false,
    )
}

fn destructive_role_hint(value: &str) -> bool {
    use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

    let normalized = value
        .to_lowercase()
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect::<String>();
    [
        "delete",
        "permanent remove",
        "remove account",
        "revoke access",
        "empty trash",
        "xoa vinh vien",
        "xoa tai khoan",
        "thu hoi quyen",
        "don sach thung rac",
    ]
    .iter()
    .any(|term| normalized.contains(term))
}

#[cfg(test)]
mod tests {
    use super::destructive_role_hint;

    #[test]
    fn recognizes_english_and_vietnamese_destructive_controls() {
        assert!(destructive_role_hint("Delete account permanently"));
        assert!(destructive_role_hint("Xóa tài khoản"));
        assert!(destructive_role_hint("Thu hồi quyền truy cập"));
        assert!(!destructive_role_hint("Open course 5"));
    }
}
