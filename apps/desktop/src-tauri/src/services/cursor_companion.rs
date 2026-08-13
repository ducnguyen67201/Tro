use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use contracts::{CursorCompanionPhase, CursorCompanionSnapshot};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    utils::config::BackgroundThrottlingPolicy,
};

pub const WINDOW_LABEL: &str = "assistant-cursor";

const ORB_WIDTH: f64 = 52.0;
const ORB_HEIGHT: f64 = 52.0;
const CARD_WIDTH: f64 = 380.0;
const CARD_HEIGHT: f64 = 190.0;
const CURSOR_GAP: f64 = 16.0;
const TRACK_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Debug, Default)]
pub struct CursorCompanion {
    runtime: Mutex<RuntimeState>,
}

#[derive(Debug, Default)]
struct RuntimeState {
    phase: CursorCompanionPhase,
    tracker: Option<Tracker>,
}

#[derive(Debug)]
struct Tracker {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl CursorCompanion {
    pub fn create_window(app: &AppHandle) -> tauri::Result<()> {
        if app.get_webview_window(WINDOW_LABEL).is_some() {
            return Ok(());
        }

        let window =
            WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("index.html".into()))
                .inner_size(ORB_WIDTH, ORB_HEIGHT)
                .transparent(true)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focusable(false)
                .shadow(false)
                .visible(false)
                .content_protected(true)
                .accept_first_mouse(true)
                .background_throttling(BackgroundThrottlingPolicy::Disabled)
                .build()?;
        window.set_ignore_cursor_events(true)?;
        #[cfg(target_os = "macos")]
        window.set_visible_on_all_workspaces(true)?;
        Ok(())
    }

    pub fn snapshot(&self) -> CursorCompanionSnapshot {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        CursorCompanionSnapshot {
            phase: runtime.phase,
        }
    }

    pub fn follow(&self, app: &AppHandle) -> tauri::Result<()> {
        {
            let runtime = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if runtime.phase == CursorCompanionPhase::Following {
                return Ok(());
            }
        }
        self.stop_tracker();

        let window = companion_window(app)?;
        configure_window(&window, false, true)?;
        position_window(app, &window, ORB_WIDTH, ORB_HEIGHT)?;
        set_phase(&self.runtime, CursorCompanionPhase::Following);
        emit_phase(&window, CursorCompanionPhase::Following);
        window.show()?;

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_app = app.clone();
        let worker_window = window.clone();
        let worker = thread::Builder::new()
            .name("tro-cursor-follow".to_owned())
            .spawn(move || track_cursor(worker_app, worker_window, worker_stop))
            .map_err(tauri::Error::Io)?;

        let mut runtime = self
            .runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.tracker = Some(Tracker {
            stop,
            worker: Some(worker),
        });
        Ok(())
    }

    pub fn anchor(&self, app: &AppHandle) -> tauri::Result<()> {
        self.stop_tracker();
        let window = companion_window(app)?;
        configure_window(&window, true, false)?;
        position_window(app, &window, CARD_WIDTH, CARD_HEIGHT)?;
        set_phase(&self.runtime, CursorCompanionPhase::Anchored);
        emit_phase(&window, CursorCompanionPhase::Anchored);
        window.show()
    }

    pub fn show_anchored_idle(&self, app: &AppHandle) -> tauri::Result<()> {
        self.anchor(app)
    }

    pub fn hide(&self, app: &AppHandle) -> tauri::Result<()> {
        self.stop_tracker();
        set_phase(&self.runtime, CursorCompanionPhase::Hidden);
        if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
            configure_window(&window, false, true)?;
            emit_phase(&window, CursorCompanionPhase::Hidden);
            window.hide()?;
        }
        Ok(())
    }

    fn stop_tracker(&self) {
        let tracker = {
            let mut runtime = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            runtime.tracker.take()
        };
        if let Some(mut tracker) = tracker {
            tracker.stop.store(true, Ordering::Release);
            if let Some(worker) = tracker.worker.take() {
                let _result = worker.join();
            }
        }
    }
}

impl Drop for CursorCompanion {
    fn drop(&mut self) {
        self.stop_tracker();
    }
}

fn companion_window(app: &AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    app.get_webview_window(WINDOW_LABEL)
        .ok_or_else(|| tauri::Error::WindowNotFound)
}

fn configure_window(
    window: &tauri::WebviewWindow,
    focusable: bool,
    ignore_cursor_events: bool,
) -> tauri::Result<()> {
    window.set_focusable(focusable)?;
    window.set_ignore_cursor_events(ignore_cursor_events)
}

fn emit_phase(window: &tauri::WebviewWindow, phase: CursorCompanionPhase) {
    if let Err(error) = window.emit(
        "cursor_companion_changed",
        CursorCompanionSnapshot { phase },
    ) {
        tracing::warn!(
            component = "cursor_companion",
            operation = "emit_phase",
            error_code = "window_event_failed",
            source = %error
        );
    }
}

fn set_phase(runtime: &Mutex<RuntimeState>, phase: CursorCompanionPhase) {
    runtime
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .phase = phase;
}

fn track_cursor(app: AppHandle, window: tauri::WebviewWindow, stop: Arc<AtomicBool>) {
    let mut previous = None;
    let mut error_logged = false;
    while !stop.load(Ordering::Acquire) {
        match desired_position(&app, ORB_WIDTH, ORB_HEIGHT) {
            Ok(position) => {
                if previous != Some(position) {
                    if let Err(error) = window.set_position(position) {
                        if !error_logged {
                            tracing::warn!(
                                component = "cursor_companion",
                                operation = "follow_cursor",
                                error_code = "window_move_failed",
                                source = %error
                            );
                            error_logged = true;
                        }
                    } else {
                        previous = Some(position);
                        error_logged = false;
                    }
                }
            }
            Err(error) => {
                if !error_logged {
                    tracing::warn!(
                        component = "cursor_companion",
                        operation = "read_cursor",
                        error_code = "cursor_position_failed",
                        source = %error
                    );
                    error_logged = true;
                }
            }
        }
        thread::sleep(TRACK_INTERVAL);
    }
}

fn position_window(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    logical_width: f64,
    logical_height: f64,
) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)?
        .or(app.primary_monitor()?)
        .ok_or(tauri::Error::FailedToReceiveMessage)?;
    let size = physical_size(logical_width, logical_height, monitor.scale_factor());
    window.set_size(size)?;
    window.set_position(place_near_cursor(
        cursor,
        size,
        monitor.work_area().position,
        monitor.work_area().size,
        physical_gap(monitor.scale_factor()),
    ))
}

fn desired_position(
    app: &AppHandle,
    logical_width: f64,
    logical_height: f64,
) -> tauri::Result<PhysicalPosition<i32>> {
    let cursor = app.cursor_position()?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)?
        .or(app.primary_monitor()?)
        .ok_or(tauri::Error::FailedToReceiveMessage)?;
    let size = physical_size(logical_width, logical_height, monitor.scale_factor());
    Ok(place_near_cursor(
        cursor,
        size,
        monitor.work_area().position,
        monitor.work_area().size,
        physical_gap(monitor.scale_factor()),
    ))
}

fn physical_size(width: f64, height: f64, scale_factor: f64) -> PhysicalSize<u32> {
    PhysicalSize::new(
        (width * scale_factor).round().max(1.0) as u32,
        (height * scale_factor).round().max(1.0) as u32,
    )
}

fn physical_gap(scale_factor: f64) -> i32 {
    (CURSOR_GAP * scale_factor).round().max(1.0) as i32
}

fn place_near_cursor(
    cursor: PhysicalPosition<f64>,
    window: PhysicalSize<u32>,
    work_position: PhysicalPosition<i32>,
    work_size: PhysicalSize<u32>,
    gap: i32,
) -> PhysicalPosition<i32> {
    let cursor_x = cursor.x.round() as i64;
    let cursor_y = cursor.y.round() as i64;
    let left = i64::from(work_position.x);
    let top = i64::from(work_position.y);
    let right = left.saturating_add(i64::from(work_size.width));
    let bottom = top.saturating_add(i64::from(work_size.height));
    let width = i64::from(window.width);
    let height = i64::from(window.height);
    let gap = i64::from(gap);

    let preferred_x = cursor_x.saturating_add(gap);
    let preferred_y = cursor_y.saturating_add(gap);
    let flipped_x = cursor_x.saturating_sub(gap).saturating_sub(width);
    let flipped_y = cursor_y.saturating_sub(gap).saturating_sub(height);
    let max_x = right.saturating_sub(width).max(left);
    let max_y = bottom.saturating_sub(height).max(top);
    let x = if preferred_x.saturating_add(width) <= right {
        preferred_x
    } else {
        flipped_x
    }
    .clamp(left, max_x);
    let y = if preferred_y.saturating_add(height) <= bottom {
        preferred_y
    } else {
        flipped_y
    }
    .clamp(top, max_y);

    PhysicalPosition::new(
        i32::try_from(x).unwrap_or(if x.is_negative() { i32::MIN } else { i32::MAX }),
        i32::try_from(y).unwrap_or(if y.is_negative() { i32::MIN } else { i32::MAX }),
    )
}

#[cfg(test)]
mod tests {
    use tauri::{PhysicalPosition, PhysicalSize};

    use super::place_near_cursor;

    #[test]
    fn prefers_below_and_right_of_cursor() {
        let position = place_near_cursor(
            PhysicalPosition::new(100.0, 100.0),
            PhysicalSize::new(52, 52),
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1000, 800),
            16,
        );
        assert_eq!(position, PhysicalPosition::new(116, 116));
    }

    #[test]
    fn flips_left_and_above_at_bottom_right_edge() {
        let position = place_near_cursor(
            PhysicalPosition::new(990.0, 790.0),
            PhysicalSize::new(100, 80),
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1000, 800),
            16,
        );
        assert_eq!(position, PhysicalPosition::new(874, 694));
    }

    #[test]
    fn places_inside_negative_origin_work_area() {
        let position = place_near_cursor(
            PhysicalPosition::new(-1915.0, 5.0),
            PhysicalSize::new(380, 190),
            PhysicalPosition::new(-1920, 0),
            PhysicalSize::new(1920, 1080),
            16,
        );
        assert_eq!(position, PhysicalPosition::new(-1899, 21));
    }

    #[test]
    fn clamps_oversized_window_to_work_area_origin() {
        let position = place_near_cursor(
            PhysicalPosition::new(50.0, 50.0),
            PhysicalSize::new(1200, 900),
            PhysicalPosition::new(-100, -50),
            PhysicalSize::new(800, 600),
            16,
        );
        assert_eq!(position, PhysicalPosition::new(-100, -50));
    }
}
