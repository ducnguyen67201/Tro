use contracts::{AppError, ErrorCode, ImageMime, PhysicalPoint, ScreenFrame, ScreenFrameMeta};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use uuid::Uuid;
use xcap::Monitor;

pub trait CaptureBackend: Send + Sync {
    fn capture_display_at(&self, cursor: Option<PhysicalPoint>) -> Result<ScreenFrame, AppError>;
}

pub struct XcapCaptureBackend;

impl CaptureBackend for XcapCaptureBackend {
    fn capture_display_at(&self, cursor: Option<PhysicalPoint>) -> Result<ScreenFrame, AppError> {
        let monitors = Monitor::all().map_err(capture_error)?;
        let monitor = cursor
            .and_then(|point| {
                monitors.iter().find(|candidate| {
                    monitor_bounds(candidate).is_some_and(|bounds| bounds.contains(point))
                })
            })
            .or_else(|| {
                monitors
                    .iter()
                    .find(|candidate| candidate.is_primary().unwrap_or(false))
            })
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
    tracing::warn!(component = "capture", operation = "capture_display_at", error_code = "capture_failed", source = %error);
    AppError::new(
        ErrorCode::CaptureFailed,
        "Tro chưa thể chụp màn hình. Hãy kiểm tra quyền ghi màn hình.",
        true,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MonitorBounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl MonitorBounds {
    fn contains(self, point: PhysicalPoint) -> bool {
        let x = i64::from(point.x);
        let y = i64::from(point.y);
        let left = i64::from(self.x);
        let top = i64::from(self.y);
        let right = left.saturating_add(i64::from(self.width));
        let bottom = top.saturating_add(i64::from(self.height));
        x >= left && x < right && y >= top && y < bottom
    }
}

fn monitor_bounds(monitor: &Monitor) -> Option<MonitorBounds> {
    Some(MonitorBounds {
        x: monitor.x().ok()?,
        y: monitor.y().ok()?,
        width: monitor.width().ok()?,
        height: monitor.height().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use contracts::PhysicalPoint;

    use super::MonitorBounds;

    #[test]
    fn includes_top_left_and_excludes_bottom_right() {
        let bounds = MonitorBounds {
            x: -1920,
            y: -200,
            width: 1920,
            height: 1080,
        };
        assert!(bounds.contains(PhysicalPoint { x: -1920, y: -200 }));
        assert!(bounds.contains(PhysicalPoint { x: -1, y: 879 }));
        assert!(!bounds.contains(PhysicalPoint { x: 0, y: 879 }));
        assert!(!bounds.contains(PhysicalPoint { x: -1, y: 880 }));
    }
}
