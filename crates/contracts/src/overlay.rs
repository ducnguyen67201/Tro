use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{NormalizedPoint, NormalizedRect};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OverlayElement {
    Rect {
        bounds: NormalizedRect,
        label: Option<String>,
    },
    Arrow {
        from: NormalizedPoint,
        to: NormalizedPoint,
        label: Option<String>,
    },
    Point {
        at: NormalizedPoint,
        label: Option<String>,
    },
    Step {
        at: NormalizedPoint,
        number: u8,
        label: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OverlayUpdate {
    pub session_id: String,
    pub generation: u64,
    pub monitor_id: String,
    pub elements: Vec<OverlayElement>,
    pub expires_after_ms: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenFrameMeta {
    pub frame_id: String,
    pub monitor_id: String,
    /// Physical target bounds used for normalized input mapping.
    pub width_px: u32,
    pub height_px: u32,
    /// Encoded image dimensions sent to the provider; these may be downscaled.
    pub image_width_px: u32,
    pub image_height_px: u32,
    pub origin_x_px: i32,
    pub origin_y_px: i32,
    pub scale_factor: f64,
    pub layout_generation: u64,
    pub mime_type: ImageMime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageMime {
    Jpeg,
    Png,
}

pub struct ScreenFrame {
    pub meta: ScreenFrameMeta,
    pub bytes: Vec<u8>,
}

impl fmt::Debug for ScreenFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScreenFrame")
            .field("meta", &self.meta)
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

impl Drop for ScreenFrame {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

pub struct CoordinateMapper;

impl CoordinateMapper {
    pub fn to_physical(point: NormalizedPoint, frame: &ScreenFrameMeta) -> PhysicalPoint {
        let x_offset = (point.x * frame.width_px.saturating_sub(1) as f32).round() as i32;
        let y_offset = (point.y * frame.height_px.saturating_sub(1) as f32).round() as i32;
        PhysicalPoint {
            x: frame.origin_x_px.saturating_add(x_offset),
            y: frame.origin_y_px.saturating_add(y_offset),
        }
    }
}
