use contracts::{
    AppError, ComputerAction, CoordinateMapper, ErrorCode, KeyCode, MouseButton, ScreenFrameMeta,
};
use enigo::{Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use tokio_util::sync::CancellationToken;

pub trait InputBackend: Send + Sync {
    fn execute(
        &self,
        action: &ComputerAction,
        frame: &ScreenFrameMeta,
        cancellation: &CancellationToken,
    ) -> Result<(), AppError>;
    fn release_all(&self) -> Result<(), AppError>;
}

pub struct NativeInputBackend;

impl InputBackend for NativeInputBackend {
    fn execute(
        &self,
        action: &ComputerAction,
        frame: &ScreenFrameMeta,
        cancellation: &CancellationToken,
    ) -> Result<(), AppError> {
        if cancellation.is_cancelled() {
            return Err(cancelled());
        }
        let mut enigo = Enigo::new(&input_settings()).map_err(input_error)?;
        match action {
            ComputerAction::Move { point } => {
                let point = CoordinateMapper::to_physical(*point, frame);
                enigo
                    .move_mouse(point.x, point.y, Coordinate::Abs)
                    .map_err(input_error)?;
            }
            ComputerAction::Click {
                point,
                button,
                count,
            } => {
                let point = CoordinateMapper::to_physical(*point, frame);
                enigo
                    .move_mouse(point.x, point.y, Coordinate::Abs)
                    .map_err(input_error)?;
                for _ in 0..*count {
                    if cancellation.is_cancelled() {
                        return Err(cancelled());
                    }
                    enigo
                        .button(map_button(*button), Direction::Click)
                        .map_err(input_error)?;
                }
            }
            ComputerAction::Scroll { delta_x, delta_y } => {
                enigo
                    .scroll(*delta_x, enigo::Axis::Horizontal)
                    .map_err(input_error)?;
                enigo
                    .scroll(*delta_y, enigo::Axis::Vertical)
                    .map_err(input_error)?;
            }
            ComputerAction::TypeText { text } => enigo.text(text.expose()).map_err(input_error)?,
            ComputerAction::Wait { milliseconds } => {
                let deadline = std::time::Instant::now()
                    + std::time::Duration::from_millis(u64::from(*milliseconds).min(10_000));
                while std::time::Instant::now() < deadline {
                    if cancellation.is_cancelled() {
                        return Err(cancelled());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            ComputerAction::Drag { from, to } => {
                let from = CoordinateMapper::to_physical(*from, frame);
                let to = CoordinateMapper::to_physical(*to, frame);
                enigo
                    .move_mouse(from.x, from.y, Coordinate::Abs)
                    .and_then(|()| enigo.button(Button::Left, Direction::Press))
                    .and_then(|()| enigo.move_mouse(to.x, to.y, Coordinate::Abs))
                    .and_then(|()| enigo.button(Button::Left, Direction::Release))
                    .map_err(input_error)?;
            }
            ComputerAction::KeyPress { keys } => {
                let mapped = keys.iter().map(map_key).collect::<Result<Vec<_>, _>>()?;
                for key in &mapped {
                    if cancellation.is_cancelled() {
                        return Err(cancelled());
                    }
                    enigo.key(*key, Direction::Press).map_err(input_error)?;
                }
                for key in mapped.iter().rev() {
                    enigo.key(*key, Direction::Release).map_err(input_error)?;
                }
            }
            ComputerAction::Capture => {}
        }
        Ok(())
    }

    fn release_all(&self) -> Result<(), AppError> {
        let mut enigo = Enigo::new(&input_settings()).map_err(input_error)?;
        for button in [Button::Left, Button::Right, Button::Middle] {
            let _result = enigo.button(button, Direction::Release);
        }
        for key in [
            enigo::Key::Control,
            enigo::Key::Alt,
            enigo::Key::Shift,
            enigo::Key::Meta,
        ] {
            let _result = enigo.key(key, Direction::Release);
        }
        Ok(())
    }
}

fn input_settings() -> Settings {
    Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    }
}

fn map_button(button: MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Right,
        MouseButton::Middle => Button::Middle,
    }
}

fn map_key(key: &KeyCode) -> Result<enigo::Key, AppError> {
    Ok(match key {
        KeyCode::Enter => enigo::Key::Return,
        KeyCode::Escape => enigo::Key::Escape,
        KeyCode::Tab => enigo::Key::Tab,
        KeyCode::Backspace => enigo::Key::Backspace,
        KeyCode::ArrowUp => enigo::Key::UpArrow,
        KeyCode::ArrowDown => enigo::Key::DownArrow,
        KeyCode::ArrowLeft => enigo::Key::LeftArrow,
        KeyCode::ArrowRight => enigo::Key::RightArrow,
        KeyCode::Control => enigo::Key::Control,
        KeyCode::Alt => enigo::Key::Alt,
        KeyCode::Shift => enigo::Key::Shift,
        KeyCode::Meta => enigo::Key::Meta,
        KeyCode::Character(value) => {
            let mut characters = value.chars();
            let character = characters.next().ok_or_else(unsupported_key)?;
            if characters.next().is_some() {
                return Err(unsupported_key());
            }
            enigo::Key::Unicode(character)
        }
    })
}

fn unsupported_key() -> AppError {
    AppError::new(
        ErrorCode::UnsupportedAction,
        "Tổ hợp phím computer use chưa được hỗ trợ.",
        false,
    )
}

fn input_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(component = "input", operation = "execute", error_code = "input_failed", source = %error);
    AppError::new(
        ErrorCode::AccessibilityPermissionDenied,
        "Tro không thể điều khiển ứng dụng này. Hãy kiểm tra quyền Trợ năng; ứng dụng chạy quyền quản trị không được hỗ trợ.",
        true,
    )
}

fn cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "Đã dừng theo yêu cầu.", false)
}

#[cfg(test)]
mod tests {
    use super::input_settings;

    #[test]
    fn input_actions_never_open_permission_prompt_implicitly() {
        assert!(!input_settings().open_prompt_to_get_permissions);
    }
}
