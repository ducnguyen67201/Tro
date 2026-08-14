use contracts::{AppError, ErrorCode};
use std::time::{Duration, Instant};

pub struct AgentLimits {
    started_at: Instant,
    turns: u32,
    actions: u32,
    consecutive_stale: u32,
    total_stale: u32,
}

impl Default for AgentLimits {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            turns: 0,
            actions: 0,
            consecutive_stale: 0,
            total_stale: 0,
        }
    }
}

impl AgentLimits {
    pub fn record_stale(&mut self) -> Result<(), AppError> {
        self.consecutive_stale = self.consecutive_stale.saturating_add(1);
        self.total_stale = self.total_stale.saturating_add(1);
        if self.consecutive_stale > 5 || self.total_stale > 10 {
            return Err(AppError::new(
                ErrorCode::AgentTurnLimit,
                "Giao diện thay đổi quá nhiều lần; Tro cần bạn thử lại.",
                false,
            ));
        }
        Ok(())
    }

    pub fn record_execution(&mut self) {
        self.consecutive_stale = 0;
    }

    pub fn total_stale(&self) -> u32 {
        self.total_stale
    }

    pub fn record_turn(&mut self, actions: u32) -> Result<(), AppError> {
        self.turns = self.turns.saturating_add(1);
        self.actions = self.actions.saturating_add(actions);
        if self.started_at.elapsed() > Duration::from_secs(300) {
            return Err(AppError::new(
                ErrorCode::AgentTimeout,
                "Phiên agent đã hết thời gian.",
                false,
            ));
        }
        if self.turns > 20 || self.actions > 100 {
            return Err(AppError::new(
                ErrorCode::AgentTurnLimit,
                "Phiên agent đã đạt giới hạn an toàn.",
                false,
            ));
        }
        Ok(())
    }
}
