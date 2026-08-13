use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

pub fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let ask = MenuItem::with_id(app, "ask", "Hỏi Tro", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Cài đặt", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Dừng khẩn cấp", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&ask, &settings, &stop, &quit])?;
    TrayIconBuilder::new()
        .tooltip("Tro — Trợ lý học tập")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _show = window.show();
                    let _focus = window.set_focus();
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
