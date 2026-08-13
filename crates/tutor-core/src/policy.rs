use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectMode {
    Math,
    English,
    Literature,
    AiLiteracy,
    General,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TutorRequestKind {
    Learning,
    AssessableWork,
    ProctoredAssessment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TutorPolicy {
    pub response_language: &'static str,
    pub first_move: &'static str,
    pub allow_final_answer: bool,
    pub may_propose_agent_goal: bool,
}

impl TutorPolicy {
    pub const fn for_request(kind: TutorRequestKind) -> Self {
        match kind {
            TutorRequestKind::Learning => Self {
                response_language: "vi-VN",
                first_move: "explain_or_hint",
                allow_final_answer: true,
                may_propose_agent_goal: true,
            },
            TutorRequestKind::AssessableWork => Self {
                response_language: "vi-VN",
                first_move: "hint_then_inspect_attempt",
                allow_final_answer: true,
                may_propose_agent_goal: false,
            },
            TutorRequestKind::ProctoredAssessment => Self {
                response_language: "vi-VN",
                first_move: "refuse_and_offer_concept_review",
                allow_final_answer: false,
                may_propose_agent_goal: false,
            },
        }
    }
}
