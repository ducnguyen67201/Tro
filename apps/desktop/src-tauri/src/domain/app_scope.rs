use std::collections::HashSet;

use contracts::{AppError, ErrorCode, ObservationBinding};
use uuid::Uuid;

use super::session::AgentLimits;

/// Local-only authority for one explicit computer-use run. The raw goal is never
/// retained here; only its digest is needed to prevent intent replacement.
pub struct RunScope {
    scope_id: Uuid,
    explicit_session_id: Uuid,
    goal_hash: blake3::Hash,
    approved_app_ids: HashSet<String>,
    target: Option<ObservationBinding>,
    pub limits: AgentLimits,
}

impl RunScope {
    pub fn new(goal: &str) -> Self {
        Self {
            scope_id: Uuid::new_v4(),
            explicit_session_id: Uuid::new_v4(),
            goal_hash: blake3::hash(goal.as_bytes()),
            approved_app_ids: HashSet::new(),
            target: None,
            limits: AgentLimits::default(),
        }
    }

    pub fn scope_id(&self) -> Uuid {
        self.scope_id
    }

    pub fn explicit_session_id(&self) -> Uuid {
        self.explicit_session_id
    }

    pub fn goal_matches(&self, goal: &str) -> bool {
        self.goal_hash == blake3::hash(goal.as_bytes())
    }

    pub fn approve_app(&mut self, app_id: impl Into<String>) {
        self.approved_app_ids.insert(app_id.into());
    }

    pub fn revoke_app(&mut self, app_id: &str) {
        self.approved_app_ids.remove(app_id);
        if self
            .target
            .as_ref()
            .is_some_and(|target| target.app_id == app_id)
        {
            self.target = None;
        }
    }

    pub fn is_app_approved(&self, app_id: &str) -> bool {
        self.approved_app_ids.contains(app_id)
    }

    pub fn bind(&mut self, binding: ObservationBinding) -> Result<(), AppError> {
        if !self.is_app_approved(&binding.app_id) {
            return Err(AppError::new(
                ErrorCode::TargetAppUnavailable,
                "Ứng dụng chưa được cho phép trong phiên computer use này.",
                false,
            ));
        }
        self.target = Some(binding);
        Ok(())
    }

    pub fn validates(&self, candidate: &ObservationBinding) -> bool {
        self.is_app_approved(&candidate.app_id)
            && self.target.as_ref().is_some_and(|target| {
                target.observation_id == candidate.observation_id
                    && target.app_id == candidate.app_id
                    && target.window_generation == candidate.window_generation
                    && target.layout_generation == candidate.layout_generation
            })
    }

    pub fn target(&self) -> Option<&ObservationBinding> {
        self.target.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use contracts::ObservationBinding;

    use super::RunScope;

    #[test]
    fn binding_requires_an_approved_exact_app() {
        let mut scope = RunScope::new("Mở khóa học số năm");
        let binding = ObservationBinding {
            observation_id: "obs-1".to_owned(),
            app_id: "app.browser".to_owned(),
            window_generation: 1,
            layout_generation: 2,
        };
        assert!(scope.bind(binding.clone()).is_err());
        scope.approve_app("app.browser");
        scope.bind(binding.clone()).expect("approved app binds");
        assert!(scope.validates(&binding));

        let stale = ObservationBinding {
            observation_id: "obs-2".to_owned(),
            ..binding
        };
        assert!(!scope.validates(&stale));
    }
}
