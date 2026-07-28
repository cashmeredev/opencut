use scene::FreeformPathPoint;
use tiny_skia::{Path, PathBuilder};

fn local_to_canvas(
    x: f64,
    y: f64,
    center_x: f64,
    center_y: f64,
    rotation: f64,
    scale: f64,
    width: f64,
    height: f64,
) -> (f64, f64) {
    let canvas_center_x = width / 2.0 + center_x * width;
    let canvas_center_y = height / 2.0 + center_y * height;
    let scaled_x = x * width * scale;
    let scaled_y = y * height * scale;
    let angle_rad = rotation.to_radians();
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    (
        canvas_center_x + scaled_x * cos - scaled_y * sin,
        canvas_center_y + scaled_x * sin + scaled_y * cos,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_freeform_path(
    points: &[FreeformPathPoint],
    center_x: f64,
    center_y: f64,
    rotation: f64,
    scale: f64,
    width: f64,
    height: f64,
    closed: bool,
) -> Option<Path> {
    if points.is_empty() {
        return None;
    }

    let to_canvas = |x: f64, y: f64| {
        local_to_canvas(x, y, center_x, center_y, rotation, scale, width, height)
    };

    let anchors: Vec<((f64, f64), (f64, f64), (f64, f64))> = points
        .iter()
        .map(|point| {
            (
                to_canvas(point.x, point.y),
                to_canvas(point.x + point.in_x, point.y + point.in_y),
                to_canvas(point.x + point.out_x, point.y + point.out_y),
            )
        })
        .collect();

    let mut builder = PathBuilder::new();
    builder.move_to(anchors[0].0 .0 as f32, anchors[0].0 .1 as f32);
    for index in 1..anchors.len() {
        let previous = anchors[index - 1];
        let current = anchors[index];
        builder.cubic_to(
            previous.2 .0 as f32,
            previous.2 .1 as f32,
            current.1 .0 as f32,
            current.1 .1 as f32,
            current.0 .0 as f32,
            current.0 .1 as f32,
        );
    }

    if closed && anchors.len() > 1 {
        let last = anchors[anchors.len() - 1];
        let first = anchors[0];
        builder.cubic_to(
            last.2 .0 as f32,
            last.2 .1 as f32,
            first.1 .0 as f32,
            first.1 .1 as f32,
            first.0 .0 as f32,
            first.0 .1 as f32,
        );
        builder.close();
    }

    builder.finish()
}
