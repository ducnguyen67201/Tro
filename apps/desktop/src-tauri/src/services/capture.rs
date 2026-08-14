use contracts::{AppError, ErrorCode, ImageMime, PhysicalPoint, ScreenFrame, ScreenFrameMeta};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use uuid::Uuid;
use xcap::{Monitor, Window};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapturePreference {
    Cursor,
    FocusedWindow,
}

pub trait CaptureBackend: Send + Sync {
    fn capture_display(
        &self,
        preference: CapturePreference,
        cursor: Option<PhysicalPoint>,
    ) -> Result<ScreenFrame, AppError>;

    fn capture_window(&self, window_id: u32) -> Result<Option<ScreenFrame>, AppError>;

    fn layout_generation(&self) -> Result<u64, AppError>;
}

pub struct XcapCaptureBackend;

impl CaptureBackend for XcapCaptureBackend {
    fn capture_display(
        &self,
        preference: CapturePreference,
        cursor: Option<PhysicalPoint>,
    ) -> Result<ScreenFrame, AppError> {
        let monitors = Monitor::all().map_err(capture_error)?;
        let focused_monitor_id = matches!(preference, CapturePreference::FocusedWindow)
            .then(focused_window_monitor_id)
            .flatten();
        let cursor_monitor_id = cursor
            .and_then(|point| {
                monitors.iter().find(|candidate| {
                    monitor_bounds(candidate).is_some_and(|bounds| bounds.contains(point))
                })
            })
            .and_then(|monitor| monitor.id().ok());
        let primary_monitor_id = monitors
            .iter()
            .find(|candidate| candidate.is_primary().unwrap_or(false))
            .and_then(|monitor| monitor.id().ok());
        let monitor_id = preferred_monitor_id(
            preference,
            focused_monitor_id,
            cursor_monitor_id,
            primary_monitor_id,
        )
        .ok_or_else(|| capture_error("no display available"))?;
        let monitor = monitors
            .iter()
            .find(|candidate| candidate.id().ok() == Some(monitor_id))
            .ok_or_else(|| capture_error("preferred display disappeared"))?;
        let image = DynamicImage::ImageRgba8(monitor.capture_image().map_err(capture_error)?);
        let encoded = encode_image(&image, ImageMime::Jpeg)?;
        Ok(ScreenFrame {
            meta: ScreenFrameMeta {
                frame_id: Uuid::new_v4().to_string(),
                monitor_id: monitor.id().map_err(capture_error)?.to_string(),
                width_px: monitor.width().map_err(capture_error)?,
                height_px: monitor.height().map_err(capture_error)?,
                image_width_px: encoded.width_px,
                image_height_px: encoded.height_px,
                origin_x_px: monitor.x().map_err(capture_error)?,
                origin_y_px: monitor.y().map_err(capture_error)?,
                scale_factor: f64::from(monitor.scale_factor().map_err(capture_error)?),
                layout_generation: display_layout_generation(&monitors),
                mime_type: encoded.mime_type,
            },
            bytes: encoded.bytes,
        })
    }

    fn capture_window(&self, window_id: u32) -> Result<Option<ScreenFrame>, AppError> {
        let windows = Window::all().map_err(capture_error)?;
        let Some(window) = windows
            .into_iter()
            .find(|candidate| candidate.id().ok() == Some(window_id))
        else {
            return Ok(None);
        };
        let width_px = window.width().map_err(capture_error)?;
        let height_px = window.height().map_err(capture_error)?;
        let Ok(captured) = window.capture_image() else {
            return Ok(None);
        };
        let image = DynamicImage::ImageRgba8(captured);
        let preferred_mime = if width_px <= 2_560 && height_px <= 2_560 {
            ImageMime::Png
        } else {
            ImageMime::Jpeg
        };
        let encoded = encode_image(&image, preferred_mime)?;
        let monitors = Monitor::all().map_err(capture_error)?;
        let scale_factor = window
            .current_monitor()
            .ok()
            .and_then(|monitor| monitor.scale_factor().ok())
            .map_or(1.0, f64::from);
        Ok(Some(ScreenFrame {
            meta: ScreenFrameMeta {
                frame_id: Uuid::new_v4().to_string(),
                monitor_id: format!("window:{window_id}"),
                width_px,
                height_px,
                image_width_px: encoded.width_px,
                image_height_px: encoded.height_px,
                origin_x_px: window.x().map_err(capture_error)?,
                origin_y_px: window.y().map_err(capture_error)?,
                scale_factor,
                layout_generation: display_layout_generation(&monitors),
                mime_type: encoded.mime_type,
            },
            bytes: encoded.bytes,
        }))
    }

    fn layout_generation(&self) -> Result<u64, AppError> {
        Monitor::all()
            .map(|monitors| display_layout_generation(&monitors))
            .map_err(capture_error)
    }
}

struct EncodedImage {
    bytes: Vec<u8>,
    width_px: u32,
    height_px: u32,
    mime_type: ImageMime,
}

fn encode_image(image: &DynamicImage, preferred_mime: ImageMime) -> Result<EncodedImage, AppError> {
    const MAX_EDGE: u32 = 3_840;
    const MAX_BYTES: usize = 6_291_456;
    let mut bounded = if image.width() > MAX_EDGE || image.height() > MAX_EDGE {
        image.resize(MAX_EDGE, MAX_EDGE, image::imageops::FilterType::Triangle)
    } else {
        image.clone()
    };
    let mut mime_type = preferred_mime;
    for _attempt in 0..8 {
        let encoded = encode_once(&bounded, mime_type)?;
        if encoded.len() <= MAX_BYTES {
            return Ok(EncodedImage {
                bytes: encoded,
                width_px: bounded.width(),
                height_px: bounded.height(),
                mime_type,
            });
        }
        mime_type = ImageMime::Jpeg;
        bounded = bounded.resize(
            (bounded.width().saturating_mul(4) / 5).max(1),
            (bounded.height().saturating_mul(4) / 5).max(1),
            image::imageops::FilterType::Triangle,
        );
    }
    Err(capture_error("captured image exceeds the byte limit"))
}

fn encode_once(image: &DynamicImage, mime_type: ImageMime) -> Result<Vec<u8>, AppError> {
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(
            &mut bytes,
            match mime_type {
                ImageMime::Jpeg => ImageFormat::Jpeg,
                ImageMime::Png => ImageFormat::Png,
            },
        )
        .map_err(capture_error)?;
    Ok(bytes.into_inner())
}

fn display_layout_generation(monitors: &[Monitor]) -> u64 {
    let mut hasher = blake3::Hasher::new();
    let mut entries = monitors
        .iter()
        .filter_map(|monitor| {
            Some((
                monitor.id().ok()?,
                monitor.x().ok()?,
                monitor.y().ok()?,
                monitor.width().ok()?,
                monitor.height().ok()?,
            ))
        })
        .collect::<Vec<_>>();
    entries.sort_unstable();
    for entry in entries {
        hasher.update(&entry.0.to_le_bytes());
        hasher.update(&entry.1.to_le_bytes());
        hasher.update(&entry.2.to_le_bytes());
        hasher.update(&entry.3.to_le_bytes());
        hasher.update(&entry.4.to_le_bytes());
    }
    let digest = hasher.finalize();
    let mut value = [0_u8; 8];
    value.copy_from_slice(&digest.as_bytes()[..8]);
    u64::from_le_bytes(value)
}

fn focused_window_monitor_id() -> Option<u32> {
    Window::all().ok()?.into_iter().find_map(|window| {
        let is_visible_and_focused =
            window.is_focused().unwrap_or(false) && !window.is_minimized().unwrap_or(true);
        is_visible_and_focused
            .then(|| window.current_monitor().ok()?.id().ok())
            .flatten()
    })
}

fn preferred_monitor_id(
    preference: CapturePreference,
    focused_monitor_id: Option<u32>,
    cursor_monitor_id: Option<u32>,
    primary_monitor_id: Option<u32>,
) -> Option<u32> {
    match preference {
        CapturePreference::Cursor => cursor_monitor_id.or(primary_monitor_id),
        CapturePreference::FocusedWindow => focused_monitor_id
            .or(cursor_monitor_id)
            .or(primary_monitor_id),
    }
}

fn capture_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(component = "capture", operation = "capture_display", error_code = "capture_failed", source = %error);
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

    use super::{CapturePreference, MonitorBounds, preferred_monitor_id};

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

    #[test]
    fn agent_capture_follows_the_focused_window_to_another_display() {
        assert_eq!(
            preferred_monitor_id(CapturePreference::FocusedWindow, Some(2), Some(1), Some(1)),
            Some(2)
        );
    }

    #[test]
    fn assistant_capture_stays_with_the_cursor_display() {
        assert_eq!(
            preferred_monitor_id(CapturePreference::Cursor, Some(2), Some(1), Some(3)),
            Some(1)
        );
    }
}
