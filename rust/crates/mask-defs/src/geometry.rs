use scene::{RectangleMaskParams, StrokeAlign};

pub const MIN_MASK_DIMENSION: f64 = 0.01;

pub const NORMAL_SNAP_EPSILON: f64 = 1e-10;
pub const MIN_POLYGON_AREA_PX: f64 = 0.5;
pub const INTERSECTION_EPSILON: f64 = 1e-6;
pub const LINE_PARALLEL_EPSILON: f64 = 1e-10;

#[derive(Clone, Copy, Debug)]
pub struct BoxGeometry {
    pub center_x: f64,
    pub center_y: f64,
    pub mask_width: f64,
    pub mask_height: f64,
    pub rotation_rad: f64,
}

pub fn box_like_geometry(params: &RectangleMaskParams, width: f64, height: f64) -> BoxGeometry {
    BoxGeometry {
        center_x: width / 2.0 + params.center_x * width,
        center_y: height / 2.0 + params.center_y * height,
        mask_width: params.width.max(MIN_MASK_DIMENSION) * width,
        mask_height: params.height.max(MIN_MASK_DIMENSION) * height,
        rotation_rad: params.rotation.to_radians(),
    }
}

pub fn stroke_offset(stroke_align: StrokeAlign, stroke_width: f64) -> f64 {
    match stroke_align {
        StrokeAlign::Inside => -(stroke_width / 2.0),
        StrokeAlign::Outside => stroke_width / 2.0,
        StrokeAlign::Center => 0.0,
    }
}

pub fn rotate_point(
    x: f64,
    y: f64,
    center_x: f64,
    center_y: f64,
    rotation_rad: f64,
) -> (f64, f64) {
    let dx = x - center_x;
    let dy = y - center_y;
    let cos = rotation_rad.cos();
    let sin = rotation_rad.sin();
    (center_x + dx * cos - dy * sin, center_y + dx * sin + dy * cos)
}

#[derive(Clone, Copy, Debug)]
pub struct SplitLine {
    pub normal_x: f64,
    pub normal_y: f64,
    pub line_x: f64,
    pub line_y: f64,
}

pub fn split_line_geometry(
    center_x: f64,
    center_y: f64,
    rotation: f64,
    width: f64,
    height: f64,
) -> SplitLine {
    let angle_rad = rotation.to_radians();
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    SplitLine {
        normal_x: if cos.abs() < NORMAL_SNAP_EPSILON { 0.0 } else { cos },
        normal_y: if sin.abs() < NORMAL_SNAP_EPSILON { 0.0 } else { sin },
        line_x: width / 2.0 + center_x * width,
        line_y: height / 2.0 + center_y * height,
    }
}

pub fn half_plane_sign(line: SplitLine, x: f64, y: f64) -> f64 {
    (x - line.line_x) * line.normal_x + (y - line.line_y) * line.normal_y
}

pub fn line_edge_intersection(
    line: SplitLine,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> Option<(f64, f64)> {
    let distance1 = half_plane_sign(line, x1, y1);
    let distance2 = half_plane_sign(line, x2, y2);
    let denom = distance1 - distance2;
    if denom.abs() < LINE_PARALLEL_EPSILON {
        return None;
    }
    let t = distance1 / denom;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    Some((x1 + (x2 - x1) * t, y1 + (y2 - y1) * t))
}

pub fn polygon_area(vertices: &[(f64, f64)]) -> f64 {
    let mut area = 0.0;
    for i in 0..vertices.len() {
        let (x1, y1) = vertices[i];
        let (x2, y2) = vertices[(i + 1) % vertices.len()];
        area += x1 * y2 - x2 * y1;
    }
    area.abs() * 0.5
}
