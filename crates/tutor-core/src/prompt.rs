use crate::SubjectMode;

pub struct PromptContext<'a> {
    pub locale: &'a str,
    pub subject: SubjectMode,
    pub age_scope_confirmed: bool,
    pub screen_context_available: bool,
}

pub fn assemble_prompt(context: &PromptContext<'_>) -> String {
    let base = include_str!("../../../prompts/tutor-vi.md");
    format!(
        "{base}\n\n<session locale=\"{}\" subject=\"{:?}\" age_18_plus=\"{}\" screen_context=\"{}\" />",
        context.locale,
        context.subject,
        context.age_scope_confirmed,
        context.screen_context_available
    )
}
