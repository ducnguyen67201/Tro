use std::{collections::HashMap, time::Instant};

use contracts::{ComputerAction, ForegroundContext};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct ConfirmationRequest {
    pub confirmation_id: String,
    pub action_vi: String,
    pub consequence_vi: String,
    pub app_name: String,
    pub expires_at_unix_ms: i64,
}

struct PendingConfirmation {
    fingerprint: blake3::Hash,
    foreground_generation: u64,
    expires_at: Instant,
}

#[derive(Default)]
pub struct ConfirmationManager {
    pending: HashMap<Uuid, PendingConfirmation>,
}

impl ConfirmationManager {
    pub fn issue(
        &mut self,
        action: &ComputerAction,
        foreground: &ForegroundContext,
    ) -> Result<ConfirmationRequest, contracts::AppError> {
        let bytes = serde_json::to_vec(action).map_err(|_| {
            contracts::AppError::new(
                contracts::ErrorCode::Internal,
                "Không thể tạo xác nhận an toàn.",
                false,
            )
        })?;
        let id = Uuid::new_v4();
        self.pending.insert(
            id,
            PendingConfirmation {
                fingerprint: blake3::hash(&bytes),
                foreground_generation: foreground.window_generation,
                expires_at: Instant::now() + std::time::Duration::from_secs(30),
            },
        );
        Ok(ConfirmationRequest {
            confirmation_id: id.to_string(),
            action_vi: describe_action(action),
            consequence_vi:
                "Thao tác này có thể gửi dữ liệu hoặc thay đổi nội dung. Chỉ cho phép đúng một lần."
                    .to_owned(),
            app_name: "Ứng dụng đang hoạt động".to_owned(),
            expires_at_unix_ms: time_now_ms().saturating_add(30_000),
        })
    }

    pub fn consume(
        &mut self,
        id: Uuid,
        action: &ComputerAction,
        foreground: &ForegroundContext,
    ) -> bool {
        let Some(pending) = self.pending.remove(&id) else {
            return false;
        };
        let Ok(bytes) = serde_json::to_vec(action) else {
            return false;
        };
        pending.expires_at > Instant::now()
            && pending.foreground_generation == foreground.window_generation
            && pending.fingerprint == blake3::hash(&bytes)
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

fn describe_action(action: &ComputerAction) -> String {
    match action {
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

fn time_now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .unwrap_or_default()
}
