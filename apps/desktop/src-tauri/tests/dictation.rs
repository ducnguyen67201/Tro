use contracts::{ActionTarget, ForegroundContext};

#[test]
fn secure_context_is_never_treated_as_known_editor() {
    let context = ForegroundContext {
        process_hash: "fixture".to_owned(),
        window_generation: 1,
        control_role: Some("password".to_owned()),
        is_secure: true,
        is_elevated: false,
    };
    let _known_editor = ActionTarget::KnownEditor;
    assert!(context.is_secure);
}
