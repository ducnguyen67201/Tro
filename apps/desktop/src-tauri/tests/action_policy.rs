use contracts::{ActionTarget, ComputerAction, ForegroundContext, NormalizedPoint, RiskTier};
use desktop_lib::security::action_policy::{ActionContext, ActionPolicy};
use desktop_lib::services::action_executor::ResolvedActionEvidence;

fn foreground() -> ForegroundContext {
    ForegroundContext {
        process_hash: "fixture".to_owned(),
        window_generation: 1,
        control_role: Some("button".to_owned()),
        is_secure: false,
        is_elevated: false,
    }
}

fn evidence() -> ResolvedActionEvidence {
    ResolvedActionEvidence {
        app_match: true,
        window_match: true,
        layout_match: true,
        secure: false,
        elevated: false,
        system_ui: false,
        role_category: Some("button".to_owned()),
        supported_operation: true,
        editable: true,
        visual_fallback: false,
        local_destructive: false,
    }
}

#[test]
fn payment_is_blocked_before_input() {
    let foreground = foreground();
    let evidence = evidence();
    let decision = ActionPolicy::evaluate(
        &ComputerAction::Move {
            point: NormalizedPoint::new(0.5, 0.5).expect("valid point"),
        },
        &ActionContext {
            explicit_session: true,
            scope_matches: true,
            app_approved: true,
            foreground: &foreground,
            target: ActionTarget::Payment,
            evidence: &evidence,
        },
    );
    assert_eq!(decision.tier, RiskTier::Blocked);
}

#[test]
fn unknown_field_requires_confirmation() {
    let foreground = foreground();
    let evidence = evidence();
    let decision = ActionPolicy::evaluate(
        &ComputerAction::Capture,
        &ActionContext {
            explicit_session: true,
            scope_matches: true,
            app_approved: true,
            foreground: &foreground,
            target: ActionTarget::UnknownField,
            evidence: &evidence,
        },
    );
    assert_eq!(decision.tier, RiskTier::Confirm);
}

#[test]
fn delete_is_blocked_even_when_provider_calls_it_benign() {
    let foreground = foreground();
    let mut evidence = evidence();
    evidence.local_destructive = true;
    let decision = ActionPolicy::evaluate(
        &ComputerAction::Capture,
        &ActionContext {
            explicit_session: true,
            scope_matches: true,
            app_approved: true,
            foreground: &foreground,
            target: ActionTarget::Benign,
            evidence: &evidence,
        },
    );
    assert_eq!(decision.tier, RiskTier::Blocked);
}
