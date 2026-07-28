use scene::SplitMaskParams;
use tiny_skia::{
    Color, GradientStop, LinearGradient, Paint, Path, PathBuilder, Pixmap, Point, SpreadMode,
    Transform,
};

use crate::geometry::{
    INTERSECTION_EPSILON, MIN_POLYGON_AREA_PX, SplitLine, half_plane_sign, line_edge_intersection,
    polygon_area, split_line_geometry,
};

fn canvas_edges(width: f64, height: f64) -> [(f64, f64, f64, f64); 4] {
    [
        (0.0, 0.0, width, 0.0),
        (width, 0.0, width, height),
        (width, height, 0.0, height),
        (0.0, height, 0.0, 0.0),
    ]
}

fn points_equal(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() <= INTERSECTION_EPSILON && (a.1 - b.1).abs() <= INTERSECTION_EPSILON
}

fn split_line(params: &SplitMaskParams, width: f64, height: f64) -> SplitLine {
    split_line_geometry(
        params.center_x,
        params.center_y,
        params.rotation,
        width,
        height,
    )
}

pub fn split_stroke_segment(
    params: &SplitMaskParams,
    width: f64,
    height: f64,
) -> Option<[(f64, f64); 2]> {
    let line = split_line(params, width, height);
    let mut intersections: Vec<(f64, f64)> = Vec::new();
    for &(x1, y1, x2, y2) in &canvas_edges(width, height) {
        let Some(hit) = line_edge_intersection(line, x1, y1, x2, y2) else {
            continue;
        };
        if intersections.iter().any(|&point| points_equal(point, hit)) {
            continue;
        }
        intersections.push(hit);
    }
    if intersections.len() != 2 {
        return None;
    }
    Some([intersections[0], intersections[1]])
}

pub fn split_stroke_path(params: &SplitMaskParams, width: f64, height: f64) -> Option<Path> {
    let segment = split_stroke_segment(params, width, height)?;
    let mut builder = PathBuilder::new();
    builder.move_to(segment[0].0 as f32, segment[0].1 as f32);
    builder.line_to(segment[1].0 as f32, segment[1].1 as f32);
    builder.finish()
}

pub fn split_opaque_body_path(params: &SplitMaskParams, width: f64, height: f64) -> Option<Path> {
    let line = split_line(params, width, height);
    let is_inside = |x: f64, y: f64| half_plane_sign(line, x, y) >= 0.0;

    let mut vertices: Vec<(f64, f64)> = Vec::new();
    for &(x1, y1, x2, y2) in &canvas_edges(width, height) {
        let vertex1_inside = is_inside(x1, y1);
        let vertex2_inside = is_inside(x2, y2);
        if vertex1_inside && vertex2_inside {
            vertices.push((x2, y2));
        } else if vertex1_inside {
            if let Some(hit) = line_edge_intersection(line, x1, y1, x2, y2) {
                vertices.push(hit);
            }
        } else if vertex2_inside {
            if let Some(hit) = line_edge_intersection(line, x1, y1, x2, y2) {
                vertices.push(hit);
                vertices.push((x2, y2));
            }
        }
    }

    if vertices.len() < 3 || polygon_area(&vertices) < MIN_POLYGON_AREA_PX {
        return None;
    }

    let mut builder = PathBuilder::new();
    builder.move_to(vertices[0].0 as f32, vertices[0].1 as f32);
    for &(x, y) in &vertices[1..] {
        builder.line_to(x as f32, y as f32);
    }
    builder.close();
    builder.finish()
}

pub fn fill_split_feather_body(
    params: &SplitMaskParams,
    width: f64,
    height: f64,
    pixmap: &mut Pixmap,
) {
    let line = split_line(params, width, height);
    let feather_half = params.feather / 2.0;
    let Some(shader) = LinearGradient::new(
        Point::from_xy(
            (line.line_x - line.normal_x * feather_half) as f32,
            (line.line_y - line.normal_y * feather_half) as f32,
        ),
        Point::from_xy(
            (line.line_x + line.normal_x * feather_half) as f32,
            (line.line_y + line.normal_y * feather_half) as f32,
        ),
        vec![
            GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, 0)),
            GradientStop::new(1.0, Color::from_rgba8(255, 255, 255, 255)),
        ],
        SpreadMode::Pad,
        Transform::default(),
    ) else {
        return;
    };
    let paint = Paint {
        shader,
        anti_alias: true,
        ..Paint::default()
    };
    let Some(rect) = tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32) else {
        return;
    };
    pixmap.fill_rect(rect, &paint, Transform::default(), None);
}
