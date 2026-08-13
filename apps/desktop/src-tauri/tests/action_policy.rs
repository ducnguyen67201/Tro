use contracts::{ActionTarget, ComputerAction, ForegroundContext, NormalizedPoint, RiskTier};
use desktop_lib::security::action_policy::{ActionContext, ActionPolicy};

fn foreground() -> ForegroundContext {
    ForegroundContext {
        process_hash: "fixture".to_owned(),
        window_generation: 1,
        control_role: Some("button".to_owned()),
        is_secure: false,
        is_elevated: false,
    }
}

#[test]
fn payment_is_blocked_before_input() {
    let foreground = foreground();
    let decision = ActionPolicy::evaluate(
        &ComputerAction::Move {
            point: NormalizedPoint::new(0.5, 0.5).expect("valid point"),
        },
        &ActionContext {
            explicit_session: true,
            goal_matches: true,
            foreground: &foreground,
            target: ActionTarget::Payment,
        },
    );
    assert_eq!(decision.tier, RiskTier::Blocked);
}

#[test]
fn unknown_field_requires_confirmation() {
    let foreground = foreground();
    let decision = ActionPolicy::evaluate(
        &ComputerAction::Capture,
        &ActionContext {
            explicit_session: true,
            goal_matches: true,
            foreground: &foreground,
            target: ActionTarget::UnknownField,
        },
    );
    assert_eq!(decision.tier, RiskTier::Confirm);
}
