use contracts::{
    ActionTarget, ComputerAction, ForegroundContext, KeyCode, PolicyDecision, PolicyReason,
    RiskTier,
};

pub struct ActionContext<'a> {
    pub explicit_session: bool,
    pub goal_matches: bool,
    pub foreground: &'a ForegroundContext,
    pub target: ActionTarget,
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
            ActionTarget::Password | ActionTarget::Otp => {
                return blocked(
                    PolicyReason::Credentials,
                    "Mật khẩu và mã xác thực luôn bị chặn.",
                );
            }
            ActionTarget::Payment | ActionTarget::Banking => {
                return blocked(
                    PolicyReason::Payment,
                    "Tro không thực hiện thanh toán hoặc giao dịch tài chính.",
                );
            }
            ActionTarget::ProctoredAssessment => {
                return blocked(
                    PolicyReason::ProctoredAssessment,
                    "Tro không thao tác trong bài thi có giám sát.",
                );
            }
            ActionTarget::PermissionOrSecurity
            | ActionTarget::Government
            | ActionTarget::Legal
            | ActionTarget::Medical => {
                return blocked(
                    PolicyReason::SafeguardChange,
                    "Thao tác nhạy cảm này cần bạn tự thực hiện.",
                );
            }
            ActionTarget::Submit
            | ActionTarget::Upload
            | ActionTarget::Delete
            | ActionTarget::Download
            | ActionTarget::Settings
            | ActionTarget::ExternalNavigation
            | ActionTarget::PersonalData
            | ActionTarget::UnknownField => {
                return confirm(
                    PolicyReason::ConsequentialAction,
                    "Tro cần bạn xác nhận đúng một thao tác này.",
                );
            }
            ActionTarget::Benign | ActionTarget::KnownEditor => {}
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
            && context.target != ActionTarget::KnownEditor
        {
            return confirm(
                PolicyReason::UnknownField,
                "Tro chưa xác định chắc chắn trường nhập liệu.",
            );
        }
        PolicyDecision {
            tier: RiskTier::Low,
            reason_code: if context.target == ActionTarget::KnownEditor {
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
