use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

use crate::{app_state::AppState, commands};

pub fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let ask = MenuItem::with_id(app, "ask", "Hiện Tro", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Cài đặt", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Dừng khẩn cấp", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&ask, &settings, &stop, &quit])?;
    TrayIconBuilder::new()
        .tooltip("Tro — Trợ lý học tập")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "ask" => {
                let state = app.state::<AppState>();
                if let Err(error) = state.cursor_companion.follow(app) {
                    tracing::warn!(
                        component = "cursor_companion",
                        operation = "show_from_tray",
                        error_code = "window_operation_failed",
                        source = %error
                    );
                }
            }
            "settings" => {
                if commands::window::show_main(app).is_ok()
                    && let Some(window) = app.get_webview_window("main")
                {
                    let _event = window.emit("open_settings", ());
                }
            }
            action => {
                let _event = app.emit("global_shortcut", action);
            }
        })
        .build(app)?;
    Ok(())
}
