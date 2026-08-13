use contracts::{CoordinateMapper, ImageMime, NormalizedPoint, ScreenFrameMeta};

#[test]
fn maps_negative_origin_and_edges_once() {
    let frame = ScreenFrameMeta {
        frame_id: "fixture".to_owned(),
        monitor_id: "left".to_owned(),
        width_px: 1920,
        height_px: 1080,
        origin_x_px: -1920,
        origin_y_px: 0,
        scale_factor: 1.25,
        layout_generation: 2,
        mime_type: ImageMime::Jpeg,
    };
    let point =
        CoordinateMapper::to_physical(NormalizedPoint::new(1.0, 1.0).expect("valid point"), &frame);
    assert_eq!(point.x, -1);
    assert_eq!(point.y, 1079);
}
