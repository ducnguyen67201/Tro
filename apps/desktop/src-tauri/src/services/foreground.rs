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
            is_secure: true,
            is_elevated: false,
        }
    }
}
