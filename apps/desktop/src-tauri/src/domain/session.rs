use contracts::{AppError, ErrorCode};
use std::time::{Duration, Instant};

pub struct AgentLimits {
    started_at: Instant,
    turns: u32,
    actions: u32,
}

impl Default for AgentLimits {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            turns: 0,
            actions: 0,
        }
    }
}

impl AgentLimits {
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
