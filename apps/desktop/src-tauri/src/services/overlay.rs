use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn create_overlays(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    for (index, monitor) in app.available_monitors()?.into_iter().enumerate() {
        let label = format!("overlay-{index}");
        if app.get_webview_window(&label).is_some() {
            continue;
        }
        let position = monitor.position();
        let size = monitor.size();
        let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
            .position(f64::from(position.x), f64::from(position.y))
            .inner_size(f64::from(size.width), f64::from(size.height))
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focusable(false)
            .shadow(false)
            .build()?;
        window.set_ignore_cursor_events(true)?;
    }
    Ok(())
}

pub fn hide_all(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with("overlay-") {
            let _hide = window.hide();
        }
    }
}
pub fn show_all(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with("overlay-") {
            let _show = window.show();
        }
    }
}
