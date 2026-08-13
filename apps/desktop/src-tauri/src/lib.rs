mod app_state;
mod commands;
pub mod domain;
pub mod platform;
pub mod security;
pub mod services;

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use app_state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new())
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                if let Err(error) = commands::window::hide_main(window.app_handle()) {
                    tracing::warn!(
                        component = "window",
                        operation = "close_to_background",
                        error_code = "window_operation_failed",
                        source = %error
                    );
                }
            }
        })
        .setup(|app| {
            register_shortcuts(app)?;
            services::hotkeys::build_tray(app)?;
            services::cursor_companion::CursorCompanion::create_window(app.handle())?;
            commands::agent::create_confirmation_window(app.handle())?;
            services::overlay::create_overlays(app.handle())?;
            let state = app.state::<AppState>();
            state.reset_after_restart();
            state.cursor_companion.follow(app.handle())?;
            #[cfg(target_os = "macos")]
            state.command_option_shortcut.ensure_started(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::assistant::get_app_snapshot,
            commands::assistant::start_assistant,
            commands::assistant::finish_assistant,
            commands::assistant::stop_assistant,
            commands::assistant::start_dictation,
            commands::assistant::stop_dictation,
            commands::agent::start_agent,
            commands::agent::resolve_confirmation,
            commands::agent::emergency_stop,
            commands::permissions::get_permission_snapshot,
            commands::permissions::request_permission,
            commands::permissions::restart_app,
            commands::settings::update_settings,
            commands::settings::get_llm_config,
            commands::window::get_cursor_companion_snapshot,
            commands::window::show_main_window,
            commands::window::hide_main_window,
            commands::window::follow_cursor_companion,
            commands::window::dismiss_cursor_companion,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(
            |error| tracing::error!(component = "desktop", operation = "run", error = %error),
        );
}

fn register_shortcuts(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let shortcuts = [
        ("CommandOrControl+Shift+D", "dictation"),
        ("CommandOrControl+Shift+Escape", "stop"),
    ];
    #[cfg(not(target_os = "macos"))]
    let shortcuts = [
        ("CommandOrControl+Shift+Space", "ask"),
        ("CommandOrControl+Shift+D", "dictation"),
        ("CommandOrControl+Shift+Escape", "stop"),
    ];
    for (shortcut, action) in shortcuts {
        let action = action.to_owned();
        app.global_shortcut()
            .on_shortcut(shortcut, move |handle, _, event| {
                if action == "ask" {
                    let emitted_action = if event.state == ShortcutState::Pressed {
                        "ask"
                    } else {
                        "ask_release"
                    };
                    let _result = handle.emit("global_shortcut", emitted_action);
                } else if event.state == ShortcutState::Pressed {
                    let _result = handle.emit("global_shortcut", action.clone());
                }
            })?;
    }
    Ok(())
}
