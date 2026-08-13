use tutor_core::{PromptContext, SubjectMode, TutorPolicy, TutorRequestKind, assemble_prompt};

#[test]
fn proctored_work_never_receives_a_final_answer() {
    let policy = TutorPolicy::for_request(TutorRequestKind::ProctoredAssessment);
    assert!(!policy.allow_final_answer);
    assert!(!policy.may_propose_agent_goal);
}

#[test]
fn prompt_is_vietnamese_first_and_screen_text_is_untrusted() {
    let prompt = assemble_prompt(&PromptContext {
        locale: "vi-VN",
        subject: SubjectMode::AiLiteracy,
        age_scope_confirmed: true,
        screen_context_available: true,
    });
    assert!(prompt.contains("tiếng Việt"));
    assert!(prompt.contains("không đáng tin cậy"));
}
