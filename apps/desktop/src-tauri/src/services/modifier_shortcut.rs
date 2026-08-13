use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use objc2_core_graphics::{CGEventSource, CGEventSourceStateID};
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL: Duration = Duration::from_millis(8);
const COMMAND_LEFT: u16 = 55;
const COMMAND_RIGHT: u16 = 54;
const OPTION_LEFT: u16 = 58;
const OPTION_RIGHT: u16 = 61;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChordTransition {
    Pressed,
    Released,
}

#[derive(Debug, Default)]
struct CommandOptionChord {
    active: bool,
}

impl CommandOptionChord {
    fn update(&mut self, state: ModifierState) -> Option<ChordTransition> {
        let next_active = state.command && state.option;
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ModifierState {
    command: bool,
    option: bool,
}

impl ModifierState {
    fn read() -> Self {
        let source = CGEventSourceStateID::CombinedSessionState;
        Self {
            command: CGEventSource::key_state(source, COMMAND_LEFT)
                || CGEventSource::key_state(source, COMMAND_RIGHT),
            option: CGEventSource::key_state(source, OPTION_LEFT)
                || CGEventSource::key_state(source, OPTION_RIGHT),
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandOptionShortcut {
    listener: Mutex<Option<Listener>>,
}

impl CommandOptionShortcut {
    /// Starts a read-only poller for the physical Command and Option key states.
    /// This check never opens a permission prompt and is safe to call repeatedly.
    pub fn ensure_started(&self, app: &AppHandle) -> bool {
        let mut listener = self
            .listener
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if listener.is_some() {
            return true;
        }

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let app = app.clone();
        let worker = match thread::Builder::new()
            .name("tro-command-option".to_owned())
            .spawn(move || listen(worker_stop, app))
        {
            Ok(worker) => worker,
            Err(error) => {
                tracing::warn!(component = "shortcut", operation = "spawn", %error);
                return false;
            }
        };

        *listener = Some(Listener {
            stop,
            worker: Some(worker),
        });
        true
    }
}

#[derive(Debug)]
struct Listener {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _result = worker.join();
        }
    }
}

fn listen(stop: Arc<AtomicBool>, app: AppHandle) {
    let mut chord = CommandOptionChord::default();
    while !stop.load(Ordering::Acquire) {
        if let Some(transition) = chord.update(ModifierState::read()) {
            handle_transition(&app, transition);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn handle_transition(app: &AppHandle, transition: ChordTransition) {
    let action = match transition {
        ChordTransition::Pressed => "ask",
        ChordTransition::Released => "ask_release",
    };
    if let Err(error) = app.emit("global_shortcut", action) {
        tracing::warn!(component = "shortcut", operation = "emit", %error);
    }
}

#[cfg(test)]
mod tests {
    use super::{ChordTransition, CommandOptionChord, ModifierState};

    #[test]
    fn activates_only_after_command_and_option_are_both_held() {
        let mut chord = CommandOptionChord::default();
        assert_eq!(
            chord.update(ModifierState {
                command: true,
                option: false,
            }),
            None
        );
        assert_eq!(
            chord.update(ModifierState {
                command: true,
                option: true,
            }),
            Some(ChordTransition::Pressed)
        );
        assert_eq!(
            chord.update(ModifierState {
                command: true,
                option: true,
            }),
            None
        );
        assert_eq!(
            chord.update(ModifierState {
                command: false,
                option: true,
            }),
            Some(ChordTransition::Released)
        );
    }

    #[test]
    fn ignores_single_modifiers() {
        let mut chord = CommandOptionChord::default();
        assert_eq!(chord.update(ModifierState::default()), None);
        assert_eq!(
            chord.update(ModifierState {
                command: false,
                option: true,
            }),
            None
        );
        assert_eq!(chord.update(ModifierState::default()), None);
    }

    #[test]
    fn emits_only_one_transition_per_edge() {
        let mut chord = CommandOptionChord::default();
        let active = ModifierState {
            command: true,
            option: true,
        };
        assert_eq!(chord.update(active), Some(ChordTransition::Pressed));
        assert_eq!(chord.update(active), None);
        assert_eq!(
            chord.update(ModifierState::default()),
            Some(ChordTransition::Released)
        );
        assert_eq!(chord.update(ModifierState::default()), None);
    }
}
