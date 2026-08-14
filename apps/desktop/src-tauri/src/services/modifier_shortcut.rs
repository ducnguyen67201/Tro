use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use contracts::{AgentState, AssistantState, AssistantUiState};
use objc2_core_graphics::{CGEventSource, CGEventSourceStateID};
use tauri::{AppHandle, Emitter, Manager};

use crate::{app_state::AppState, commands};

const POLL_INTERVAL: Duration = Duration::from_millis(8);
const COMMAND_LEFT: u16 = 55;
const COMMAND_RIGHT: u16 = 54;
const CONTROL_LEFT: u16 = 59;
const CONTROL_RIGHT: u16 = 62;
const ESCAPE: u16 = 53;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChordTransition {
    Pressed,
    Released,
}

#[derive(Debug, Default)]
struct CommandControlChord {
    active: bool,
}

#[derive(Debug, Default)]
struct PressEdge {
    pressed: bool,
}

impl PressEdge {
    fn update(&mut self, pressed: bool) -> bool {
        let just_pressed = pressed && !self.pressed;
        self.pressed = pressed;
        just_pressed
    }
}

impl CommandControlChord {
    fn update(&mut self, state: ModifierState) -> Option<ChordTransition> {
        let next_active = state.command && state.control;
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
    control: bool,
}

impl ModifierState {
    fn read() -> Self {
        let source = CGEventSourceStateID::CombinedSessionState;
        Self {
            command: CGEventSource::key_state(source, COMMAND_LEFT)
                || CGEventSource::key_state(source, COMMAND_RIGHT),
            control: CGEventSource::key_state(source, CONTROL_LEFT)
                || CGEventSource::key_state(source, CONTROL_RIGHT),
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandControlShortcut {
    listener: Mutex<Option<Listener>>,
}

impl CommandControlShortcut {
    /// Starts a read-only poller for the physical Command and Control key states.
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
            .name("tro-command-control".to_owned())
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
    let mut chord = CommandControlChord::default();
    let mut escape = PressEdge::default();
    while !stop.load(Ordering::Acquire) {
        if let Some(transition) = chord.update(ModifierState::read()) {
            handle_transition(&app, transition);
        }
        if escape.update(key_pressed(ESCAPE)) {
            handle_emergency_stop(&app);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn key_pressed(key_code: u16) -> bool {
    CGEventSource::key_state(CGEventSourceStateID::CombinedSessionState, key_code)
}

fn handle_transition(app: &AppHandle, transition: ChordTransition) {
    let state = app.state::<AppState>();
    let agent_active = {
        let snapshot = state
            .snapshot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        agent_is_active(snapshot.agent)
    };
    if agent_active {
        state.user_activity.record_physical_activity();
        state.cancellation().cancel();
        let _release = state.action_executor.release_all();
        return;
    }
    let action = match transition {
        ChordTransition::Pressed => "ask",
        ChordTransition::Released => "ask_release",
    };
    if let Err(error) = app.emit("global_shortcut", action) {
        tracing::warn!(component = "shortcut", operation = "emit", %error);
    }
}

fn handle_emergency_stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let active = {
        let snapshot = state
            .snapshot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        work_is_active(&snapshot)
    };
    if !active {
        return;
    }
    if let Err(error) = commands::agent::emergency_stop_with_state(app, &state) {
        tracing::warn!(
            component = "shortcut",
            operation = "escape_emergency_stop",
            error_code = ?error.code
        );
    }
}

fn work_is_active(snapshot: &AssistantUiState) -> bool {
    matches!(
        snapshot.assistant,
        AssistantState::Capturing
            | AssistantState::Listening
            | AssistantState::Thinking
            | AssistantState::Speaking
            | AssistantState::Guiding
    ) || agent_is_active(snapshot.agent)
}

fn agent_is_active(agent: AgentState) -> bool {
    matches!(
        agent,
        AgentState::ResolvingApp
            | AgentState::AwaitingAppApproval
            | AgentState::ActivatingApp
            | AgentState::Planning
            | AgentState::Validating
            | AgentState::AwaitingConfirmation
            | AgentState::Executing
            | AgentState::Stabilizing
            | AgentState::Observing
            | AgentState::StaleRecovery
    )
}

#[cfg(test)]
mod tests {
    use contracts::{AgentState, AssistantState, AssistantUiState};

    use super::{ChordTransition, CommandControlChord, ModifierState, PressEdge, work_is_active};

    #[test]
    fn activates_only_after_command_and_control_are_both_held() {
        let mut chord = CommandControlChord::default();
        assert_eq!(
            chord.update(ModifierState {
                command: true,
                control: false,
            }),
            None
        );
        assert_eq!(
            chord.update(ModifierState {
                command: true,
                control: true,
            }),
            Some(ChordTransition::Pressed)
        );
        assert_eq!(
            chord.update(ModifierState {
                command: true,
                control: true,
            }),
            None
        );
        assert_eq!(
            chord.update(ModifierState {
                command: false,
                control: true,
            }),
            Some(ChordTransition::Released)
        );
    }

    #[test]
    fn ignores_single_modifiers() {
        let mut chord = CommandControlChord::default();
        assert_eq!(chord.update(ModifierState::default()), None);
        assert_eq!(
            chord.update(ModifierState {
                command: false,
                control: true,
            }),
            None
        );
        assert_eq!(chord.update(ModifierState::default()), None);
    }

    #[test]
    fn releases_when_either_modifier_is_lifted() {
        for released in [
            ModifierState {
                command: false,
                control: true,
            },
            ModifierState {
                command: true,
                control: false,
            },
        ] {
            let mut chord = CommandControlChord::default();
            assert_eq!(
                chord.update(ModifierState {
                    command: true,
                    control: true,
                }),
                Some(ChordTransition::Pressed)
            );
            assert_eq!(chord.update(released), Some(ChordTransition::Released));
        }
    }

    #[test]
    fn emits_only_one_transition_per_edge() {
        let mut chord = CommandControlChord::default();
        let active = ModifierState {
            command: true,
            control: true,
        };
        assert_eq!(chord.update(active), Some(ChordTransition::Pressed));
        assert_eq!(chord.update(active), None);
        assert_eq!(
            chord.update(ModifierState::default()),
            Some(ChordTransition::Released)
        );
        assert_eq!(chord.update(ModifierState::default()), None);
    }

    #[test]
    fn escape_emits_only_once_until_the_key_is_released() {
        let mut edge = PressEdge::default();
        assert!(!edge.update(false));
        assert!(edge.update(true));
        assert!(!edge.update(true));
        assert!(!edge.update(false));
        assert!(edge.update(true));
    }

    #[test]
    fn emergency_stop_runs_only_while_assistant_or_agent_work_is_active() {
        let mut snapshot = AssistantUiState::default();
        assert!(!work_is_active(&snapshot));

        snapshot.assistant = AssistantState::Thinking;
        assert!(work_is_active(&snapshot));

        snapshot.assistant = AssistantState::Idle;
        snapshot.agent = AgentState::Executing;
        assert!(work_is_active(&snapshot));

        snapshot.agent = AgentState::Completed;
        assert!(!work_is_active(&snapshot));
    }
}
