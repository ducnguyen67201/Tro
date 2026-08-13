use contracts::{
    AppError, ComputerAction, CoordinateMapper, ErrorCode, MouseButton, ScreenFrameMeta,
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
        let mut enigo = Enigo::new(&Settings::default()).map_err(input_error)?;
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
            ComputerAction::Capture => {}
            ComputerAction::Drag { .. } | ComputerAction::KeyPress { .. } => {
                return Err(AppError::new(
                    ErrorCode::UnsupportedAction,
                    "Thao tác nhập này chưa được hỗ trợ an toàn.",
                    false,
                ));
            }
        }
        Ok(())
    }

    fn release_all(&self) -> Result<(), AppError> {
        let mut enigo = Enigo::new(&Settings::default()).map_err(input_error)?;
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

fn map_button(button: MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Right,
        MouseButton::Middle => Button::Middle,
    }
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
