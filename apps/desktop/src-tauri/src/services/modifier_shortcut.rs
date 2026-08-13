use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use keytap::{EventKind, Key, RecvTimeoutError, Tap};
use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChordTransition {
    Pressed,
    Released,
}

#[derive(Debug, Default)]
struct CommandOptionChord {
    meta_left: bool,
    meta_right: bool,
    alt_left: bool,
    alt_right: bool,
    active: bool,
}

impl CommandOptionChord {
    fn update(&mut self, event: EventKind) -> Option<ChordTransition> {
        match event {
            EventKind::KeyDown(key) => self.set_key(key, true),
            EventKind::KeyUp(key) => self.set_key(key, false),
            EventKind::KeyRepeat(_) => return None,
        }

        let next_active = (self.meta_left || self.meta_right) && (self.alt_left || self.alt_right);
        if next_active == self.active {
            return None;
        }
        self.active = next_active;
        Some(if next_active {
            ChordTransition::Pressed
        } else {
            ChordTransition::Released
        })
    }

    fn set_key(&mut self, key: Key, pressed: bool) {
        match key {
            Key::MetaLeft => self.meta_left = pressed,
            Key::MetaRight => self.meta_right = pressed,
            Key::AltLeft => self.alt_left = pressed,
            Key::AltRight => self.alt_right = pressed,
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandOptionShortcut {
    listener: Mutex<Option<Listener>>,
}

impl CommandOptionShortcut {
    /// Starts the observe-only keyboard listener if macOS has granted Input Monitoring.
    /// This check never opens a permission prompt and is safe to call repeatedly.
    pub fn ensure_started(&self, app: &AppHandle) -> bool {
        let mut listener = self
            .listener
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if listener.is_some() {
            return true;
        }

        let tap = match Tap::new() {
            Ok(tap) => Arc::new(tap),
            Err(error) => {
                tracing::debug!(component = "shortcut", operation = "start", %error);
                return false;
            }
        };
        let stop = Arc::new(AtomicBool::new(false));
        let worker_tap = Arc::clone(&tap);
        let worker_stop = Arc::clone(&stop);
        let app = app.clone();
        let worker = match thread::Builder::new()
            .name("tro-command-option".to_owned())
            .spawn(move || listen(worker_tap, worker_stop, app))
        {
            Ok(worker) => worker,
            Err(error) => {
                tracing::warn!(component = "shortcut", operation = "spawn", %error);
                return false;
            }
        };

        *listener = Some(Listener {
            tap,
            stop,
            worker: Some(worker),
        });
        true
    }
}

#[derive(Debug)]
struct Listener {
    tap: Arc<Tap>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _result = worker.join();
        }
        // Keep the tap alive until the consumer thread has exited. Dropping the
        // final Arc then stops and joins keytap's native event-tap thread.
        let _tap = &self.tap;
    }
}

fn listen(tap: Arc<Tap>, stop: Arc<AtomicBool>, app: AppHandle) {
    let mut chord = CommandOptionChord::default();
    while !stop.load(Ordering::Acquire) {
        match tap.recv_timeout(POLL_INTERVAL) {
            Ok(event) => {
                if let Some(transition) = chord.update(event.kind) {
                    let state = app.state::<AppState>();
                    let companion_result = match transition {
                        ChordTransition::Pressed => state.cursor_companion.follow(&app),
                        ChordTransition::Released => state.cursor_companion.anchor(&app),
                    };
                    if let Err(error) = companion_result {
                        tracing::warn!(
                            component = "cursor_companion",
                            operation = "modifier_transition",
                            error_code = "window_operation_failed",
                            source = %error
                        );
                    }
                    let action = match transition {
                        ChordTransition::Pressed => "ask",
                        ChordTransition::Released => "ask_release",
                    };
                    if let Err(error) = app.emit("global_shortcut", action) {
                        tracing::warn!(component = "shortcut", operation = "emit", %error);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use keytap::{EventKind, Key};

    use super::{ChordTransition, CommandOptionChord};

    #[test]
    fn activates_only_after_command_and_option_are_both_held() {
        let mut chord = CommandOptionChord::default();
        assert_eq!(chord.update(EventKind::KeyDown(Key::MetaLeft)), None);
        assert_eq!(
            chord.update(EventKind::KeyDown(Key::AltLeft)),
            Some(ChordTransition::Pressed)
        );
        assert_eq!(chord.update(EventKind::KeyRepeat(Key::AltLeft)), None);
        assert_eq!(
            chord.update(EventKind::KeyUp(Key::MetaLeft)),
            Some(ChordTransition::Released)
        );
    }

    #[test]
    fn accepts_mixed_left_and_right_modifier_keys() {
        let mut chord = CommandOptionChord::default();
        assert_eq!(chord.update(EventKind::KeyDown(Key::MetaRight)), None);
        assert_eq!(
            chord.update(EventKind::KeyDown(Key::AltLeft)),
            Some(ChordTransition::Pressed)
        );
        assert_eq!(chord.update(EventKind::KeyDown(Key::AltRight)), None);
        assert_eq!(chord.update(EventKind::KeyUp(Key::AltLeft)), None);
        assert_eq!(
            chord.update(EventKind::KeyUp(Key::AltRight)),
            Some(ChordTransition::Released)
        );
    }

    #[test]
    fn ignores_unrelated_keys_and_duplicate_edges() {
        let mut chord = CommandOptionChord::default();
        assert_eq!(chord.update(EventKind::KeyDown(Key::A)), None);
        assert_eq!(chord.update(EventKind::KeyDown(Key::MetaLeft)), None);
        assert_eq!(chord.update(EventKind::KeyDown(Key::MetaLeft)), None);
        assert_eq!(
            chord.update(EventKind::KeyDown(Key::AltRight)),
            Some(ChordTransition::Pressed)
        );
        assert_eq!(chord.update(EventKind::KeyUp(Key::A)), None);
    }
}
