use contracts::ForegroundContext;

pub trait ForegroundContextBackend: Send + Sync {
    fn snapshot(&self) -> ForegroundContext;
}
pub struct PlatformForegroundBackend;
impl ForegroundContextBackend for PlatformForegroundBackend {
    fn snapshot(&self) -> ForegroundContext {
        ForegroundContext {
            process_hash: "unknown".to_owned(),
            window_generation: 0,
            control_role: None,
            // Sensitive targets are still blocked by the action target policy. The
            // platform backend does not currently expose a reliable secure-field bit.
            is_secure: false,
            is_elevated: false,
        }
    }
}
