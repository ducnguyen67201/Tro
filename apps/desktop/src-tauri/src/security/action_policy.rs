use contracts::{
    ActionTarget, ComputerAction, ForegroundContext, KeyCode, PolicyDecision, PolicyReason,
    RiskTier,
};

use crate::services::action_executor::ResolvedActionEvidence;

pub struct ActionContext<'a> {
    pub explicit_session: bool,
    pub scope_matches: bool,
    pub app_approved: bool,
    pub foreground: &'a ForegroundContext,
    pub target: ActionTarget,
    pub evidence: &'a ResolvedActionEvidence,
}

pub struct ActionPolicy;
impl ActionPolicy {
    pub fn evaluate(action: &ComputerAction, context: &ActionContext<'_>) -> PolicyDecision {
        if !context.explicit_session || !context.scope_matches {
            return blocked(
                PolicyReason::GoalMismatch,
                "Thao tác nằm ngoài mục tiêu đã xác nhận.",
            );
        }
        if !context.app_approved || !context.evidence.app_match {
            return blocked(
                PolicyReason::UnapprovedApplication,
                "Ứng dụng không nằm trong phạm vi đã được cho phép.",
            );
        }
        if !context.evidence.window_match || !context.evidence.layout_match {
            return blocked(
                PolicyReason::StaleObservation,
                "Cửa sổ đã thay đổi; Tro phải quan sát lại.",
            );
        }
        if context.foreground.is_elevated || context.evidence.elevated || context.evidence.system_ui
        {
            return blocked(
                PolicyReason::PrivilegeChange,
                "Tro không điều khiển ứng dụng chạy quyền quản trị.",
            );
        }
        if context.foreground.is_secure || context.evidence.secure {
            return blocked(
                PolicyReason::SensitiveField,
                "Trường bảo mật không bao giờ được tự động điền.",
            );
        }
        let target = escalated_target(context.target, context.evidence.local_target);
        match target {
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
            ActionTarget::Delete => {
                return blocked(
                    PolicyReason::DestructiveAction,
                    "Tro không xóa vĩnh viễn dữ liệu trong chế độ này.",
                );
            }
            ActionTarget::Submit
            | ActionTarget::Upload
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
        if context.evidence.local_destructive {
            return blocked(
                PolicyReason::DestructiveAction,
                "Tro không thực hiện thao tác xóa hoặc gỡ bỏ vĩnh viễn.",
            );
        }
        if !context.evidence.supported_operation {
            return blocked(
                PolicyReason::UnsupportedAction,
                "Ứng dụng không hỗ trợ thao tác này trên phần tử đã chọn.",
            );
        }
        if let ComputerAction::KeyPress { keys } = action
            && keys.iter().any(|key| matches!(key, KeyCode::Enter))
        {
            return confirm(
                PolicyReason::ConsequentialAction,
                "Phím Enter có thể gửi hoặc xác nhận nội dung.",
            );
        }
        if matches!(
            action,
            ComputerAction::TypeText { .. }
                | ComputerAction::Element {
                    operation: contracts::ElementOperationKind::SetValue,
                    ..
                }
        ) && (target != ActionTarget::KnownEditor || !context.evidence.editable)
        {
            return confirm(
                PolicyReason::UnknownField,
                "Tro chưa xác định chắc chắn trường nhập liệu.",
            );
        }
        if context.evidence.visual_fallback
            && matches!(
                action,
                ComputerAction::Click { .. } | ComputerAction::Drag { .. }
            )
        {
            return confirm(
                PolicyReason::UnknownField,
                "Tro cần xác nhận vì thao tác này dùng vị trí trên hình ảnh.",
            );
        }
        PolicyDecision {
            tier: RiskTier::Low,
            reason_code: if target == ActionTarget::KnownEditor {
                PolicyReason::KnownEditor
            } else {
                PolicyReason::BenignNavigation
            },
            display_vi: "Thao tác ít rủi ro trong phiên agent đang hoạt động.".to_owned(),
        }
    }
}

fn escalated_target(provider: ActionTarget, local: ActionTarget) -> ActionTarget {
    if target_floor(local) > target_floor(provider) {
        local
    } else {
        provider
    }
}

const fn target_floor(target: ActionTarget) -> u8 {
    match target {
        ActionTarget::Benign | ActionTarget::KnownEditor => 0,
        ActionTarget::UnknownField
        | ActionTarget::Submit
        | ActionTarget::Upload
        | ActionTarget::Download
        | ActionTarget::Settings
        | ActionTarget::ExternalNavigation
        | ActionTarget::PersonalData => 1,
        ActionTarget::Delete
        | ActionTarget::Password
        | ActionTarget::Otp
        | ActionTarget::Payment
        | ActionTarget::Banking
        | ActionTarget::Legal
        | ActionTarget::Medical
        | ActionTarget::Government
        | ActionTarget::ProctoredAssessment
        | ActionTarget::PermissionOrSecurity => 2,
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
