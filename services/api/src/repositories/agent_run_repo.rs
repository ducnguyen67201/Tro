use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Active,
    Completed,
    Stopped,
}

impl RunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentRunRecord {
    pub id: Uuid,
    pub device_id: Uuid,
    pub continuation_encrypted: Vec<u8>,
    pub status: RunStatus,
    pub turn_count: u32,
    pub action_count: u32,
    pub expires_at: OffsetDateTime,
    pub last_idempotency_key: Option<String>,
    pub last_response: Option<Vec<u8>>,
}
