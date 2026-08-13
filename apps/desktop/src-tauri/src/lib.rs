mod app_state;
mod commands;
pub mod domain;
pub mod platform;
pub mod security;
pub mod services;

use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use app_state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new())
        .setup(|app| {
            register_shortcuts(app)?;
            services::hotkeys::build_tray(app)?;
            services::overlay::create_overlays(app.handle())?;
            use tauri::Manager;
            app.state::<AppState>().reset_after_restart();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::assistant::get_app_snapshot,
            commands::assistant::start_assistant,
            commands::assistant::stop_assistant,
            commands::assistant::start_dictation,
            commands::assistant::stop_dictation,
            commands::agent::start_agent,
            commands::agent::resolve_confirmation,
            commands::agent::emergency_stop,
            commands::permissions::get_permission_snapshot,
            commands::permissions::request_permission,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(
            |error| tracing::error!(component = "desktop", operation = "run", error = %error),
        );
}

fn register_shortcuts(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcuts = [
        ("CommandOrControl+Shift+Space", "ask"),
        ("CommandOrControl+Shift+D", "dictation"),
        ("CommandOrControl+Shift+Escape", "stop"),
    ];
    for (shortcut, action) in shortcuts {
        let action = action.to_owned();
        app.global_shortcut()
            .on_shortcut(shortcut, move |handle, _, event| {
                if event.state == ShortcutState::Pressed {
                    let _result = handle.emit("global_shortcut", action.clone());
                }
            })?;
    }
    Ok(())
}
