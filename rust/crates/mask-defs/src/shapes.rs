use scene::RectangleMaskParams;
use tiny_skia::{Path, PathBuilder, Transform};

use crate::geometry::{box_like_geometry, rotate_point, stroke_offset};

const STAR_INNER_RADIUS_RATIO: f64 = 0.45;
const STAR_VERTEX_COUNT: usize = 10;

fn polygon_path(points: &[(f64, f64)]) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let first = points.first()?;
    builder.move_to(first.0 as f32, first.1 as f32);
    for point in &points[1..] {
        builder.line_to(point.0 as f32, point.1 as f32);
    }
    builder.close();
    builder.finish()
}

fn rotated_rect_path(
    center_x: f64,
    center_y: f64,
    half_width: f64,
    half_height: f64,
    rotation_rad: f64,
) -> Option<Path> {
    let corners = [
        (center_x - half_width, center_y - half_height),
        (center_x + half_width, center_y - half_height),
        (center_x + half_width, center_y + half_height),
        (center_x - half_width, center_y + half_height),
    ];
    let rotated: Vec<(f64, f64)> = corners
        .iter()
        .map(|&(x, y)| rotate_point(x, y, center_x, center_y, rotation_rad))
        .collect();
    polygon_path(&rotated)
}

fn rotated_ellipse_path(
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation_rad: f64,
) -> Option<Path> {
    let rect = tiny_skia::Rect::from_ltrb(
        -radius_x as f32,
        -radius_y as f32,
        radius_x as f32,
        radius_y as f32,
    )?;
    let path = PathBuilder::from_oval(rect)?;
    let cos = rotation_rad.cos() as f32;
    let sin = rotation_rad.sin() as f32;
    path.transform(Transform::from_row(
        cos,
        sin,
        -sin,
        cos,
        center_x as f32,
        center_y as f32,
    ))
}

fn heart_path(
    center_x: f64,
    center_y: f64,
    half_width: f64,
    half_height: f64,
    rotation_rad: f64,
) -> Option<Path> {
    let to_point = |local_x: f64, local_y: f64| {
        rotate_point(
            center_x + local_x,
            center_y + local_y,
            center_x,
            center_y,
            rotation_rad,
        )
    };

    let start = to_point(0.0, -half_height * 0.475);
    let right_control1 = to_point(half_width, -half_height * 1.225);
    let right_control2 = to_point(half_width, -half_height * 0.125);
    let bottom = to_point(0.0, half_height * 0.725);
    let left_control1 = to_point(-half_width, -half_height * 0.125);
    let left_control2 = to_point(-half_width, -half_height * 1.225);

    let mut builder = PathBuilder::new();
    builder.move_to(start.0 as f32, start.1 as f32);
    builder.cubic_to(
        right_control1.0 as f32,
        right_control1.1 as f32,
        right_control2.0 as f32,
        right_control2.1 as f32,
        bottom.0 as f32,
        bottom.1 as f32,
    );
    builder.cubic_to(
        left_control1.0 as f32,
        left_control1.1 as f32,
        left_control2.0 as f32,
        left_control2.1 as f32,
        start.0 as f32,
        start.1 as f32,
    );
    builder.close();
    builder.finish()
}

fn diamond_path(
    center_x: f64,
    center_y: f64,
    half_width: f64,
    half_height: f64,
    rotation_rad: f64,
) -> Option<Path> {
    let points = [
        (center_x, center_y - half_height),
        (center_x + half_width, center_y),
        (center_x, center_y + half_height),
        (center_x - half_width, center_y),
    ];
    let rotated: Vec<(f64, f64)> = points
        .iter()
        .map(|&(x, y)| rotate_point(x, y, center_x, center_y, rotation_rad))
        .collect();
    polygon_path(&rotated)
}

fn star_path(
    center_x: f64,
    center_y: f64,
    half_width: f64,
    half_height: f64,
    rotation_rad: f64,
) -> Option<Path> {
    let mut points = Vec::with_capacity(STAR_VERTEX_COUNT);
    for index in 0..STAR_VERTEX_COUNT {
        let is_outer = index % 2 == 0;
        let ratio = if is_outer { 1.0 } else { STAR_INNER_RADIUS_RATIO };
        let angle = (index as f64 * std::f64::consts::PI) / 5.0 - std::f64::consts::FRAC_PI_2;
        let x = center_x + half_width * ratio * angle.cos();
        let y = center_y + half_height * ratio * angle.sin();
        points.push(rotate_point(x, y, center_x, center_y, rotation_rad));
    }
    polygon_path(&points)
}

fn box_like_body_path(
    params: &RectangleMaskParams,
    width: f64,
    height: f64,
    build: fn(f64, f64, f64, f64, f64) -> Option<Path>,
) -> Option<Path> {
    let geometry = box_like_geometry(params, width, height);
    build(
        geometry.center_x,
        geometry.center_y,
        geometry.mask_width / 2.0,
        geometry.mask_height / 2.0,
        geometry.rotation_rad,
    )
}

fn box_like_stroke_path(
    params: &RectangleMaskParams,
    width: f64,
    height: f64,
    build: fn(f64, f64, f64, f64, f64) -> Option<Path>,
) -> Option<Path> {
    let geometry = box_like_geometry(params, width, height);
    let offset = stroke_offset(params.stroke_align, params.stroke_width);
    build(
        geometry.center_x,
        geometry.center_y,
        (geometry.mask_width / 2.0 + offset).max(1.0),
        (geometry.mask_height / 2.0 + offset).max(1.0),
        geometry.rotation_rad,
    )
}

pub fn rectangle_body_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_body_path(params, width, height, rotated_rect_path)
}

pub fn rectangle_stroke_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_stroke_path(params, width, height, rotated_rect_path)
}

pub fn ellipse_body_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_body_path(params, width, height, rotated_ellipse_path)
}

pub fn ellipse_stroke_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_stroke_path(params, width, height, rotated_ellipse_path)
}

pub fn heart_body_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_body_path(params, width, height, heart_path)
}

pub fn heart_stroke_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_stroke_path(params, width, height, heart_path)
}

pub fn diamond_body_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_body_path(params, width, height, diamond_path)
}

pub fn diamond_stroke_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_stroke_path(params, width, height, diamond_path)
}

pub fn star_body_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_body_path(params, width, height, star_path)
}

pub fn star_stroke_path(params: &RectangleMaskParams, width: f64, height: f64) -> Option<Path> {
    box_like_stroke_path(params, width, height, star_path)
}

fn cinematic_bars_dimensions(params: &RectangleMaskParams, width: f64, height: f64) -> (f64, f64) {
    (
        (params.width * width).max(width),
        params.height.max(0.01) * height,
    )
}

pub fn cinematic_bars_body_path(
    params: &RectangleMaskParams,
    width: f64,
    height: f64,
) -> Option<Path> {
    let (mask_width, mask_height) = cinematic_bars_dimensions(params, width, height);
    rotated_rect_path(
        width / 2.0 + params.center_x * width,
        height / 2.0 + params.center_y * height,
        mask_width / 2.0,
        mask_height / 2.0,
        params.rotation.to_radians(),
    )
}

pub fn cinematic_bars_stroke_path(
    params: &RectangleMaskParams,
    width: f64,
    height: f64,
) -> Option<Path> {
    let (mask_width, mask_height) = cinematic_bars_dimensions(params, width, height);
    let offset = stroke_offset(params.stroke_align, params.stroke_width);
    rotated_rect_path(
        width / 2.0 + params.center_x * width,
        height / 2.0 + params.center_y * height,
        (mask_width / 2.0 + offset).max(1.0),
        (mask_height / 2.0 + offset).max(1.0),
        params.rotation.to_radians(),
    )
}
