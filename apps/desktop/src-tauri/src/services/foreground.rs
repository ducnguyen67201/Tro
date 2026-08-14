use contracts::ForegroundContext;
use xcap::Window;

pub trait ForegroundContextBackend: Send + Sync {
    fn snapshot(&self) -> ForegroundContext;
}
pub struct PlatformForegroundBackend;
impl ForegroundContextBackend for PlatformForegroundBackend {
    fn snapshot(&self) -> ForegroundContext {
        let focused = Window::all().ok().and_then(|windows| {
            windows.into_iter().find(|window| {
                window.is_focused().unwrap_or(false) && !window.is_minimized().unwrap_or(true)
            })
        });
        let Some(window) = focused else {
            return ForegroundContext {
                process_hash: "unavailable".to_owned(),
                window_generation: 0,
                control_role: None,
                is_secure: true,
                is_elevated: false,
            };
        };
        let app_name = window.app_name().unwrap_or_default();
        let mut hasher = blake3::Hasher::new();
        hasher.update(app_name.as_bytes());
        hasher.update(&window.id().unwrap_or_default().to_le_bytes());
        hasher.update(&window.x().unwrap_or_default().to_le_bytes());
        hasher.update(&window.y().unwrap_or_default().to_le_bytes());
        let digest = hasher.finalize();
        let mut generation = [0_u8; 8];
        generation.copy_from_slice(&digest.as_bytes()[..8]);
        ForegroundContext {
            process_hash: blake3::hash(app_name.as_bytes()).to_hex().to_string(),
            window_generation: u64::from_le_bytes(generation),
            control_role: None,
            // Compatibility-only path: action execution uses the observation
            // adapter and local element evidence instead of this snapshot.
            is_secure: false,
            is_elevated: false,
        }
    }
}
