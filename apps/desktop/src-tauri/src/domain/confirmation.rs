use std::{collections::HashMap, time::Instant};

use contracts::{ApplicationRef, ComputerAction, ObservationBinding, PlannedComputerAction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationKind {
    AppAccess,
    ConsequentialAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationChoice {
    AllowOnce,
    AlwaysAllow,
    Stop,
}

#[derive(Clone, Debug, Serialize)]
pub struct ConfirmationRequest {
    pub confirmation_id: String,
    pub kind: ConfirmationKind,
    pub action_vi: String,
    pub consequence_vi: String,
    pub app_name: String,
    pub identity_summary: String,
    pub choices: Vec<ConfirmationChoice>,
    pub expires_at_unix_ms: i64,
}

struct PendingConfirmation {
    fingerprint: blake3::Hash,
    kind: ConfirmationKind,
    expires_at: Instant,
}

#[derive(Default)]
pub struct ConfirmationManager {
    pending: HashMap<Uuid, PendingConfirmation>,
}

impl ConfirmationManager {
    pub fn issue_action(
        &mut self,
        scope_id: Uuid,
        planned: &PlannedComputerAction,
        binding: &ObservationBinding,
        app: &ApplicationRef,
    ) -> Result<ConfirmationRequest, contracts::AppError> {
        let fingerprint = action_fingerprint(scope_id, planned, binding)?;
        let id = Uuid::new_v4();
        self.pending.insert(
            id,
            PendingConfirmation {
                fingerprint,
                kind: ConfirmationKind::ConsequentialAction,
                expires_at: Instant::now() + std::time::Duration::from_secs(30),
            },
        );
        Ok(ConfirmationRequest {
            confirmation_id: id.to_string(),
            kind: ConfirmationKind::ConsequentialAction,
            action_vi: describe_action(&planned.action),
            consequence_vi:
                "Thao tác này có thể gửi dữ liệu hoặc thay đổi nội dung. Chỉ cho phép đúng một lần."
                    .to_owned(),
            app_name: app.display_name.clone(),
            identity_summary: app.identity_summary.clone(),
            choices: vec![ConfirmationChoice::AllowOnce, ConfirmationChoice::Stop],
            expires_at_unix_ms: time_now_ms().saturating_add(30_000),
        })
    }

    pub fn issue_app_access(
        &mut self,
        app: &ApplicationRef,
    ) -> Result<ConfirmationRequest, contracts::AppError> {
        let id = Uuid::new_v4();
        let fingerprint = app_fingerprint(app)?;
        self.pending.insert(
            id,
            PendingConfirmation {
                fingerprint,
                kind: ConfirmationKind::AppAccess,
                expires_at: Instant::now() + std::time::Duration::from_secs(30),
            },
        );
        Ok(ConfirmationRequest {
            confirmation_id: id.to_string(),
            kind: ConfirmationKind::AppAccess,
            action_vi: "Cho phép Tro dùng ứng dụng này".to_owned(),
            consequence_vi: "Quyền ứng dụng không cho phép Tro gửi, xóa hoặc thay đổi bảo mật."
                .to_owned(),
            app_name: app.display_name.clone(),
            identity_summary: app.identity_summary.clone(),
            choices: vec![
                ConfirmationChoice::AllowOnce,
                ConfirmationChoice::AlwaysAllow,
                ConfirmationChoice::Stop,
            ],
            expires_at_unix_ms: time_now_ms().saturating_add(30_000),
        })
    }

    pub fn consume_action(
        &mut self,
        id: Uuid,
        scope_id: Uuid,
        planned: &PlannedComputerAction,
        binding: &ObservationBinding,
    ) -> bool {
        let Some(pending) = self.pending.remove(&id) else {
            return false;
        };
        let Ok(fingerprint) = action_fingerprint(scope_id, planned, binding) else {
            return false;
        };
        pending.expires_at > Instant::now()
            && pending.kind == ConfirmationKind::ConsequentialAction
            && pending.fingerprint == fingerprint
    }

    pub fn consume_app(&mut self, id: Uuid, app: &ApplicationRef) -> bool {
        let Some(pending) = self.pending.remove(&id) else {
            return false;
        };
        let Ok(fingerprint) = app_fingerprint(app) else {
            return false;
        };
        pending.expires_at > Instant::now()
            && pending.kind == ConfirmationKind::AppAccess
            && pending.fingerprint == fingerprint
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

fn describe_action(action: &ComputerAction) -> String {
    match action {
        ComputerAction::ActivateApplication { .. } => "Mở ứng dụng đã chọn",
        ComputerAction::Element { operation, .. } => match operation {
            contracts::ElementOperationKind::SetValue => "Nhập nội dung vào trường đã chọn",
            _ => "Thao tác phần tử giao diện đã chọn",
        },
        ComputerAction::TypeText { .. } => "Nhập văn bản vào trường đang chọn",
        ComputerAction::Click { .. } => "Bấm nút đang được chỉ định",
        ComputerAction::KeyPress { .. } => "Nhấn tổ hợp phím",
        ComputerAction::Drag { .. } => "Kéo nội dung trên màn hình",
        ComputerAction::Scroll { .. } => "Cuộn nội dung",
        ComputerAction::Move { .. } => "Di chuyển con trỏ",
        ComputerAction::Wait { .. } => "Chờ ứng dụng phản hồi",
        ComputerAction::Capture => "Chụp lại màn hình",
    }
    .to_owned()
}

#[derive(Serialize)]
struct ActionFingerprint<'a> {
    scope_id: Uuid,
    observation_id: &'a str,
    app_id: &'a str,
    window_generation: u64,
    layout_generation: u64,
    locator: &'a contracts::ActionLocator,
    action: &'a ComputerAction,
}

fn action_fingerprint(
    scope_id: Uuid,
    planned: &PlannedComputerAction,
    binding: &ObservationBinding,
) -> Result<blake3::Hash, contracts::AppError> {
    let material = ActionFingerprint {
        scope_id,
        observation_id: &planned.observation_id,
        app_id: &binding.app_id,
        window_generation: binding.window_generation,
        layout_generation: binding.layout_generation,
        locator: &planned.locator,
        action: &planned.action,
    };
    serde_json::to_vec(&material)
        .map(|bytes| blake3::hash(&bytes))
        .map_err(|_| confirmation_error())
}

fn app_fingerprint(app: &ApplicationRef) -> Result<blake3::Hash, contracts::AppError> {
    serde_json::to_vec(&(app.app_id.as_str(), app.identity_summary.as_str()))
        .map(|bytes| blake3::hash(&bytes))
        .map_err(|_| confirmation_error())
}

fn confirmation_error() -> contracts::AppError {
    contracts::AppError::new(
        contracts::ErrorCode::Internal,
        "Không thể tạo xác nhận an toàn.",
        false,
    )
}

fn time_now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use contracts::{
        ActionLocator, ActionTarget, ApplicationRef, ComputerAction, ObservationBinding,
        PlannedComputerAction,
    };

    use super::ConfirmationManager;

    #[test]
    fn action_confirmation_is_once_and_bound_to_the_exact_observation() {
        let mut manager = ConfirmationManager::default();
        let app = ApplicationRef {
            app_id: "app".to_owned(),
            display_name: "ABC Browser".to_owned(),
            identity_summary: "fixture".to_owned(),
        };
        let binding = ObservationBinding {
            observation_id: "obs-1".to_owned(),
            app_id: "app".to_owned(),
            window_generation: 1,
            layout_generation: 1,
        };
        let planned = PlannedComputerAction {
            observation_id: "obs-1".to_owned(),
            locator: ActionLocator::Frame,
            action: ComputerAction::Capture,
            target: ActionTarget::Submit,
            description_vi: "Gửi".to_owned(),
        };
        let scope = uuid::Uuid::new_v4();
        let request = manager
            .issue_action(scope, &planned, &binding, &app)
            .expect("confirmation issued");
        let id = uuid::Uuid::parse_str(&request.confirmation_id).expect("confirmation ID");
        let changed = ObservationBinding {
            window_generation: 2,
            ..binding.clone()
        };
        assert!(!manager.consume_action(id, scope, &planned, &changed));

        let request = manager
            .issue_action(scope, &planned, &binding, &app)
            .expect("confirmation reissued");
        let id = uuid::Uuid::parse_str(&request.confirmation_id).expect("confirmation ID");
        assert!(manager.consume_action(id, scope, &planned, &binding));
        assert!(!manager.consume_action(id, scope, &planned, &binding));
    }
}
