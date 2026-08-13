#[derive(Clone, Copy, Debug, Default)]
pub struct UsageSnapshot {
    pub realtime_seconds: u32,
    pub screenshots: u32,
    pub agent_turns: u32,
}
