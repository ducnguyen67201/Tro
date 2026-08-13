use contracts::{
    ComputerAction, ForegroundContext, KeyCode, PolicyDecision, PolicyReason, RiskTier,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetHint {
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

pub struct ActionContext<'a> {
    pub explicit_session: bool,
    pub goal_matches: bool,
    pub foreground: &'a ForegroundContext,
    pub target: TargetHint,
}

pub struct ActionPolicy;
impl ActionPolicy {
    pub fn evaluate(action: &ComputerAction, context: &ActionContext<'_>) -> PolicyDecision {
        if !context.explicit_session || !context.goal_matches {
            return blocked(
                PolicyReason::GoalMismatch,
                "Thao tác nằm ngoài mục tiêu đã xác nhận.",
            );
        }
        if context.foreground.is_elevated {
            return blocked(
                PolicyReason::PrivilegeChange,
                "Tro không điều khiển ứng dụng chạy quyền quản trị.",
            );
        }
        if context.foreground.is_secure {
            return blocked(
                PolicyReason::SensitiveField,
                "Trường bảo mật không bao giờ được tự động điền.",
            );
        }
        match context.target {
            TargetHint::Password | TargetHint::Otp => {
                return blocked(
                    PolicyReason::Credentials,
                    "Mật khẩu và mã xác thực luôn bị chặn.",
                );
            }
            TargetHint::Payment | TargetHint::Banking => {
                return blocked(
                    PolicyReason::Payment,
                    "Tro không thực hiện thanh toán hoặc giao dịch tài chính.",
                );
            }
            TargetHint::ProctoredAssessment => {
                return blocked(
                    PolicyReason::ProctoredAssessment,
                    "Tro không thao tác trong bài thi có giám sát.",
                );
            }
            TargetHint::PermissionOrSecurity
            | TargetHint::Government
            | TargetHint::Legal
            | TargetHint::Medical => {
                return blocked(
                    PolicyReason::SafeguardChange,
                    "Thao tác nhạy cảm này cần bạn tự thực hiện.",
                );
            }
            TargetHint::Submit
            | TargetHint::Upload
            | TargetHint::Delete
            | TargetHint::Download
            | TargetHint::Settings
            | TargetHint::ExternalNavigation
            | TargetHint::PersonalData
            | TargetHint::UnknownField => {
                return confirm(
                    PolicyReason::ConsequentialAction,
                    "Tro cần bạn xác nhận đúng một thao tác này.",
                );
            }
            TargetHint::Benign | TargetHint::KnownEditor => {}
        }
        if let ComputerAction::KeyPress { keys } = action
            && keys.iter().any(|key| matches!(key, KeyCode::Enter))
        {
            return confirm(
                PolicyReason::ConsequentialAction,
                "Phím Enter có thể gửi hoặc xác nhận nội dung.",
            );
        }
        if let ComputerAction::TypeText { .. } = action
            && context.target != TargetHint::KnownEditor
        {
            return confirm(
                PolicyReason::UnknownField,
                "Tro chưa xác định chắc chắn trường nhập liệu.",
            );
        }
        PolicyDecision {
            tier: RiskTier::Low,
            reason_code: if context.target == TargetHint::KnownEditor {
                PolicyReason::KnownEditor
            } else {
                PolicyReason::BenignNavigation
            },
            display_vi: "Thao tác ít rủi ro trong phiên agent đang hoạt động.".to_owned(),
        }
    }
}

fn confirm(reason_code: PolicyReason, message: &str) -> PolicyDecision {
    PolicyDecision {
        tier: RiskTier::Confirm,
        reason_code,
        display_vi: message.to_owned(),
    }
}
fn blocked(reason_code: PolicyReason, message: &str) -> PolicyDecision {
    PolicyDecision {
        tier: RiskTier::Blocked,
        reason_code,
        display_vi: message.to_owned(),
    }
}
