use contracts::{AppError, ErrorCode, ImageMime, ScreenFrame, ScreenFrameMeta};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use uuid::Uuid;
use xcap::Monitor;

pub trait CaptureBackend: Send + Sync {
    fn capture_active_display(&self) -> Result<ScreenFrame, AppError>;
}

pub struct XcapCaptureBackend;

impl CaptureBackend for XcapCaptureBackend {
    fn capture_active_display(&self) -> Result<ScreenFrame, AppError> {
        let monitors = Monitor::all().map_err(capture_error)?;
        let monitor = monitors
            .into_iter()
            .find(|candidate| candidate.is_primary().unwrap_or(false))
            .ok_or_else(|| capture_error("no primary display"))?;
        let image = monitor.capture_image().map_err(capture_error)?;
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, ImageFormat::Jpeg)
            .map_err(capture_error)?;
        Ok(ScreenFrame {
            meta: ScreenFrameMeta {
                frame_id: Uuid::new_v4().to_string(),
                monitor_id: monitor.id().map_err(capture_error)?.to_string(),
                width_px: monitor.width().map_err(capture_error)?,
                height_px: monitor.height().map_err(capture_error)?,
                origin_x_px: monitor.x().map_err(capture_error)?,
                origin_y_px: monitor.y().map_err(capture_error)?,
                scale_factor: f64::from(monitor.scale_factor().map_err(capture_error)?),
                layout_generation: 0,
                mime_type: ImageMime::Jpeg,
            },
            bytes: bytes.into_inner(),
        })
    }
}

fn capture_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(component = "capture", operation = "capture_active_display", error_code = "capture_failed", source = %error);
    AppError::new(
        ErrorCode::CaptureFailed,
        "Tro chưa thể chụp màn hình. Hãy kiểm tra quyền ghi màn hình.",
        true,
    )
}
