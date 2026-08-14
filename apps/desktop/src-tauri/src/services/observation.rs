use std::{fmt, sync::Arc};

use contracts::{
    AppError, ApplicationRef, CaptureScope, ErrorCode, ForegroundContext, ObservationBinding,
    ScreenFrame, UiObservationMetadata,
};
use uuid::Uuid;
use xcap::Window;
use zeroize::Zeroizing;

use crate::{
    domain::observation::ObservationRegistry,
    services::capture::{CaptureBackend, CapturePreference},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationMode {
    Full,
    Lightweight,
}

pub struct Observation {
    pub metadata: UiObservationMetadata,
    pub frame: Option<ScreenFrame>,
    pub registry: ObservationRegistry,
    pub foreground: ForegroundContext,
    digest: blake3::Hash,
}

impl Observation {
    pub fn from_parts(
        metadata: UiObservationMetadata,
        frame: Option<ScreenFrame>,
        registry: ObservationRegistry,
        foreground: ForegroundContext,
    ) -> Self {
        let digest = observation_digest(&metadata, frame.as_ref());
        Self {
            metadata,
            frame,
            registry,
            foreground,
            digest,
        }
    }

    pub fn digest(&self) -> blake3::Hash {
        self.digest
    }

    pub fn serialized_metadata(&self) -> Result<Zeroizing<Vec<u8>>, AppError> {
        serde_json::to_vec(&self.metadata)
            .map(Zeroizing::new)
            .map_err(|_| observation_error("metadata serialization failed"))
    }
}

impl fmt::Debug for Observation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Observation")
            .field("binding", &self.metadata.binding)
            .field("capture_scope", &self.metadata.capture_scope)
            .field("element_count", &self.metadata.elements.len())
            .field("content", &"[REDACTED]")
            .finish()
    }
}

pub trait ObservationBackend: Send + Sync {
    fn observe(&self, app: &ApplicationRef, mode: ObservationMode)
    -> Result<Observation, AppError>;
}

pub struct PlatformObservationBackend {
    capture: Arc<dyn CaptureBackend>,
}

impl PlatformObservationBackend {
    pub fn new(capture: Arc<dyn CaptureBackend>) -> Self {
        Self { capture }
    }
}

impl ObservationBackend for PlatformObservationBackend {
    fn observe(
        &self,
        app: &ApplicationRef,
        mode: ObservationMode,
    ) -> Result<Observation, AppError> {
        let windows = Window::all().map_err(observation_error)?;
        let selected = select_window(&windows, &app.display_name).ok_or_else(|| {
            AppError::new(
                ErrorCode::TargetAppUnavailable,
                "Tro chưa tìm thấy cửa sổ phù hợp của ứng dụng đã chọn.",
                true,
            )
        })?;
        let window_id = selected.id().map_err(observation_error)?;
        let window_generation = window_generation(selected, &app.app_id)?;
        // Lightweight samples still capture the exact window so visual-only apps
        // have a meaningful digest. They are never uploaded; only the final stable
        // full observation leaves the desktop process.
        let exact_frame = self.capture.capture_window(window_id)?;
        let (frame, capture_scope) = if let Some(frame) = exact_frame {
            (Some(frame), CaptureScope::ExactWindow)
        } else if mode == ObservationMode::Full {
            (
                Some(
                    self.capture
                        .capture_display(CapturePreference::FocusedWindow, None)?,
                ),
                CaptureScope::MonitorFallback,
            )
        } else {
            (None, CaptureScope::SemanticOnly)
        };
        let layout_generation = frame.as_ref().map_or_else(
            || self.capture.layout_generation(),
            |captured| Ok(captured.meta.layout_generation),
        )?;
        let binding = ObservationBinding {
            observation_id: Uuid::new_v4().to_string(),
            app_id: app.app_id.clone(),
            window_generation,
            layout_generation,
        };
        // Native AX/UIA snapshots populate this collection when available. Empty is
        // an explicit semantic-degradation signal; it never grants visual autonomy.
        let metadata = UiObservationMetadata {
            binding: binding.clone(),
            capture_scope,
            elements: Vec::new(),
            truncated: false,
        };
        let registry = ObservationRegistry::new(binding.clone(), []);
        let foreground = ForegroundContext {
            process_hash: blake3::hash(app.app_id.as_bytes()).to_hex().to_string(),
            window_generation,
            control_role: None,
            is_secure: false,
            is_elevated: false,
        };
        Ok(Observation::from_parts(
            metadata, frame, registry, foreground,
        ))
    }
}

fn select_window<'a>(windows: &'a [Window], display_name: &str) -> Option<&'a Window> {
    windows
        .iter()
        .filter(|window| {
            window
                .app_name()
                .is_ok_and(|name| name.eq_ignore_ascii_case(display_name))
                && !window.is_minimized().unwrap_or(true)
                && platform_window_allowed(window)
        })
        .max_by_key(|window| {
            (
                window.is_focused().unwrap_or(false),
                u64::from(window.width().unwrap_or_default())
                    * u64::from(window.height().unwrap_or_default()),
            )
        })
}

#[cfg(target_os = "macos")]
fn platform_window_allowed(_window: &Window) -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
fn platform_window_allowed(window: &Window) -> bool {
    window.is_focused().unwrap_or(false)
}

fn window_generation(window: &Window, app_id: &str) -> Result<u64, AppError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(app_id.as_bytes());
    hasher.update(&window.id().map_err(observation_error)?.to_le_bytes());
    hasher.update(&window.x().map_err(observation_error)?.to_le_bytes());
    hasher.update(&window.y().map_err(observation_error)?.to_le_bytes());
    hasher.update(&window.width().map_err(observation_error)?.to_le_bytes());
    hasher.update(&window.height().map_err(observation_error)?.to_le_bytes());
    let digest = hasher.finalize();
    let mut value = [0_u8; 8];
    value.copy_from_slice(&digest.as_bytes()[..8]);
    Ok(u64::from_le_bytes(value))
}

fn observation_digest(
    metadata: &UiObservationMetadata,
    frame: Option<&ScreenFrame>,
) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&metadata.binding.window_generation.to_le_bytes());
    hasher.update(&metadata.binding.layout_generation.to_le_bytes());
    hasher.update(&(metadata.elements.len() as u64).to_le_bytes());
    if let Some(frame) = frame {
        hasher.update(&frame.meta.width_px.to_le_bytes());
        hasher.update(&frame.meta.height_px.to_le_bytes());
        hasher.update(&blake3::hash(&frame.bytes).as_bytes()[..8]);
    }
    hasher.finalize()
}

fn observation_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "observation",
        operation = "observe_target",
        error_code = "capture_failed",
        source = %error
    );
    AppError::new(
        ErrorCode::CaptureFailed,
        "Tro chưa thể quan sát cửa sổ ứng dụng an toàn.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use contracts::{CaptureScope, ObservationBinding, UiObservationMetadata};

    use super::observation_digest;

    #[test]
    fn binding_changes_are_visible_to_the_stabilizer_digest() {
        let metadata = UiObservationMetadata {
            binding: ObservationBinding {
                observation_id: "one".to_owned(),
                app_id: "app".to_owned(),
                window_generation: 1,
                layout_generation: 1,
            },
            capture_scope: CaptureScope::SemanticOnly,
            elements: Vec::new(),
            truncated: false,
        };
        let first = observation_digest(&metadata, None);
        let changed = UiObservationMetadata {
            binding: ObservationBinding {
                window_generation: 2,
                ..metadata.binding.clone()
            },
            ..metadata
        };
        assert_ne!(first, observation_digest(&changed, None));
    }
}
