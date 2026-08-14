use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivitySnapshot {
    generation: u64,
    platform_generation: u64,
}

pub trait UserActivityBackend: Send + Sync {
    fn snapshot(&self) -> ActivitySnapshot;
    fn changed_since(&self, snapshot: ActivitySnapshot) -> bool;
    fn begin_synthetic_input(&self) -> SyntheticInputLease<'_>;
}

pub struct NativeUserActivityBackend {
    physical_generation: AtomicU64,
    synthetic_depth: AtomicUsize,
    platform_monitor: bool,
}

impl NativeUserActivityBackend {
    /// Deterministic backend for fake-port tests. Production should use `default()`.
    pub fn manual() -> Self {
        Self {
            physical_generation: AtomicU64::new(0),
            synthetic_depth: AtomicUsize::new(0),
            platform_monitor: false,
        }
    }

    /// Called by the platform keyboard/pointer monitor. Events that arrive while
    /// Tro is injecting input are ignored; the lease is intentionally very short.
    pub fn record_physical_activity(&self) {
        if self.synthetic_depth.load(Ordering::Acquire) == 0 {
            self.physical_generation.fetch_add(1, Ordering::AcqRel);
        }
    }
}

impl Default for NativeUserActivityBackend {
    fn default() -> Self {
        Self {
            physical_generation: AtomicU64::new(0),
            synthetic_depth: AtomicUsize::new(0),
            platform_monitor: true,
        }
    }
}

impl UserActivityBackend for NativeUserActivityBackend {
    fn snapshot(&self) -> ActivitySnapshot {
        ActivitySnapshot {
            generation: self.physical_generation.load(Ordering::Acquire),
            platform_generation: self
                .platform_monitor
                .then(platform_activity_generation)
                .unwrap_or_default(),
        }
    }

    fn changed_since(&self, snapshot: ActivitySnapshot) -> bool {
        self.physical_generation.load(Ordering::Acquire) != snapshot.generation
            || (self.platform_monitor
                && platform_activity_generation() != snapshot.platform_generation)
    }

    fn begin_synthetic_input(&self) -> SyntheticInputLease<'_> {
        self.synthetic_depth.fetch_add(1, Ordering::AcqRel);
        SyntheticInputLease {
            depth: &self.synthetic_depth,
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_activity_generation() -> u64 {
    use objc2_core_graphics::{CGEventSource, CGEventSourceStateID, CGEventType};

    let mut hasher = blake3::Hasher::new();
    for event_type in [
        CGEventType::LeftMouseDown,
        CGEventType::RightMouseDown,
        CGEventType::OtherMouseDown,
        CGEventType::MouseMoved,
        CGEventType::LeftMouseDragged,
        CGEventType::RightMouseDragged,
        CGEventType::OtherMouseDragged,
        CGEventType::ScrollWheel,
        CGEventType::KeyDown,
        CGEventType::FlagsChanged,
    ] {
        hasher.update(
            &CGEventSource::counter_for_event_type(
                CGEventSourceStateID::HIDSystemState,
                event_type,
            )
            .to_le_bytes(),
        );
    }
    let digest = hasher.finalize();
    let mut generation = [0_u8; 8];
    generation.copy_from_slice(&digest.as_bytes()[..8]);
    u64::from_le_bytes(generation)
}

#[cfg(target_os = "windows")]
fn platform_activity_generation() -> u64 {
    use std::mem::size_of;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    let mut info = LASTINPUTINFO {
        cbSize: u32::try_from(size_of::<LASTINPUTINFO>()).unwrap_or(u32::MAX),
        dwTime: 0,
    };
    // SAFETY: `info` is initialized with the required structure size and remains
    // exclusively borrowed for the duration of the synchronous Win32 call.
    if unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        u64::from(info.dwTime)
    } else {
        0
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_activity_generation() -> u64 {
    0
}

pub struct SyntheticInputLease<'a> {
    depth: &'a AtomicUsize,
}

impl Drop for SyntheticInputLease<'_> {
    fn drop(&mut self) {
        self.depth.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeUserActivityBackend, UserActivityBackend};

    #[test]
    fn synthetic_input_does_not_look_like_takeover() {
        let activity = NativeUserActivityBackend::manual();
        let before = activity.snapshot();
        {
            let _lease = activity.begin_synthetic_input();
            activity.record_physical_activity();
        }
        assert!(!activity.changed_since(before));
        activity.record_physical_activity();
        assert!(activity.changed_since(before));
    }
}
