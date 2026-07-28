use scene::params::ParamValues;
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Point, Rect, Shader, Transform};

use crate::color::parse_css_color;
use crate::error::GraphicsError;
use crate::params::{ParamDefinition, ParamGroup, SelectOption};
use crate::stroke::apply_aligned_stroke;
use crate::types::{GraphicDefinition, param_number, param_string};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeAlign {
    Inside,
    Center,
    Outside,
}

impl StrokeAlign {
    fn parse(value: &str) -> StrokeAlign {
        match value {
            "inside" => StrokeAlign::Inside,
            "outside" => StrokeAlign::Outside,
            _ => StrokeAlign::Center,
        }
    }
}

pub const STROKE_ALIGN_PARAM: ParamDefinition = ParamDefinition::Select {
    key: "strokeAlign",
    label: "Stroke align",
    group: Some(ParamGroup::Stroke),
    default: "center",
    options: &[
        SelectOption { value: "inside", label: "Inside" },
        SelectOption { value: "center", label: "Center" },
        SelectOption { value: "outside", label: "Outside" },
    ],
};

const FILL_PARAM: ParamDefinition = ParamDefinition::Color {
    key: "fill",
    label: "Fill",
    group: None,
    default: "#ffffff",
};

const STROKE_PARAM: ParamDefinition = ParamDefinition::Color {
    key: "stroke",
    label: "Color",
    group: Some(ParamGroup::Stroke),
    default: "#000000",
};

const STROKE_WIDTH_PARAM: ParamDefinition = ParamDefinition::Number {
    key: "strokeWidth",
    label: "Width",
    group: Some(ParamGroup::Stroke),
    default: 0.0,
    min: 0.0,
    max: Some(64.0),
    step: 1.0,
    short_label: Some("W"),
};

const CORNER_RADIUS_PARAM: ParamDefinition = ParamDefinition::Number {
    key: "cornerRadius",
    label: "Corner radius",
    group: None,
    default: 0.0,
    min: 0.0,
    max: Some(50.0),
    step: 1.0,
    short_label: Some("R"),
};

struct ShapeParams {
    fill: Color,
    stroke: Color,
    stroke_width: f64,
    stroke_align: StrokeAlign,
    inset: f64,
}

fn resolve_shape_params(params: &ParamValues) -> Result<ShapeParams, GraphicsError> {
    let fill_raw = param_string(params, "fill", "#ffffff");
    let stroke_raw = param_string(params, "stroke", "#000000");
    let fill = parse_css_color(&fill_raw)
        .ok_or_else(|| GraphicsError::InvalidColor(fill_raw.clone()))?;
    let stroke = parse_css_color(&stroke_raw)
        .ok_or_else(|| GraphicsError::InvalidColor(stroke_raw.clone()))?;
    let stroke_width = param_number(params, "strokeWidth", 0.0).max(0.0);
    let stroke_align = StrokeAlign::parse(&param_string(params, "strokeAlign", "center"));
    let inset = if stroke_align == StrokeAlign::Center {
        stroke_width / 2.0
    } else {
        0.0
    };
    Ok(ShapeParams {
        fill: Color::from_rgba(
            fill.r as f32,
            fill.g as f32,
            fill.b as f32,
            fill.a as f32,
        )
        .unwrap_or(Color::TRANSPARENT),
        stroke: Color::from_rgba(
            stroke.r as f32,
            stroke.g as f32,
            stroke.b as f32,
            stroke.a as f32,
        )
        .unwrap_or(Color::TRANSPARENT),
        stroke_width,
        stroke_align,
        inset,
    })
}

fn fill_and_stroke(
    pixmap: &mut tiny_skia::Pixmap,
    path: &Path,
    params: &ShapeParams,
) {
    let paint = Paint {
        shader: Shader::SolidColor(params.fill),
        anti_alias: true,
        ..Paint::default()
    };
    pixmap.fill_path(path, &paint, FillRule::Winding, Transform::default(), None);
    if params.stroke_width > 0.0 {
        apply_aligned_stroke(
            pixmap,
            path,
            params.stroke_width,
            params.stroke_align,
            params.stroke,
        );
    }
}

fn push_round_rect(builder: &mut PathBuilder, x: f64, y: f64, w: f64, h: f64, radius: f64) {
    if radius <= 0.0 {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
            builder.push_rect(rect);
        }
        return;
    }
    let radius = radius.min(w / 2.0).min(h / 2.0);
    const KAPPA: f64 = 0.552_284_749_830_793_6;
    let c = radius * KAPPA;
    let (x, y, w, h, r) = (x as f32, y as f32, w as f32, h as f32, radius as f32);
    let c = c as f32;
    builder.move_to(x + r, y);
    builder.line_to(x + w - r, y);
    builder.cubic_to(x + w - r + c, y, x + w, y + r - c, x + w, y + r);
    builder.line_to(x + w, y + h - r);
    builder.cubic_to(x + w, y + h - r + c, x + w - r + c, y + h, x + w - r, y + h);
    builder.line_to(x + r, y + h);
    builder.cubic_to(x + r - c, y + h, x, y + h - r + c, x, y + h - r);
    builder.line_to(x, y + r);
    builder.cubic_to(x, y + r - c, x + r - c, y, x + r, y);
    builder.close();
}

fn arc_to(
    builder: &mut PathBuilder,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    radius: f64,
) {
    let Some(last) = builder.last_point() else {
        builder.move_to(x1 as f32, y1 as f32);
        return;
    };
    let (x0, y0) = (f64::from(last.x), f64::from(last.y));
    let coincident = |ax: f64, ay: f64, bx: f64, by: f64| ax == bx && ay == by;
    if radius <= 0.0
        || coincident(x0, y0, x1, y1)
        || coincident(x1, y1, x2, y2)
    {
        builder.line_to(x1 as f32, y1 as f32);
        return;
    }
    let v1 = (x0 - x1, y0 - y1);
    let v2 = (x2 - x1, y2 - y1);
    let cross = v1.0 * v2.1 - v1.1 * v2.0;
    if cross == 0.0 {
        builder.line_to(x1 as f32, y1 as f32);
        return;
    }
    let len1 = v1.0.hypot(v1.1);
    let len2 = v2.0.hypot(v2.1);
    let u1 = (v1.0 / len1, v1.1 / len1);
    let u2 = (v2.0 / len2, v2.1 / len2);
    let cos_angle = (u1.0 * u2.0 + u1.1 * u2.1).clamp(-1.0, 1.0);
    let angle = cos_angle.acos();
    if angle == 0.0 || angle == std::f64::consts::PI {
        builder.line_to(x1 as f32, y1 as f32);
        return;
    }
    let tangent = radius / (angle / 2.0).tan();
    let t1 = (x1 + u1.0 * tangent, y1 + u1.1 * tangent);
    let t2 = (x1 + u2.0 * tangent, y1 + u2.1 * tangent);
    builder.line_to(t1.0 as f32, t1.1 as f32);
    let bisector = (u1.0 + u2.0, u1.1 + u2.1);
    let bisector_len = bisector.0.hypot(bisector.1);
    if bisector_len == 0.0 {
        return;
    }
    let center_dist = radius / (angle / 2.0).sin();
    let center = (
        x1 + bisector.0 / bisector_len * center_dist,
        y1 + bisector.1 / bisector_len * center_dist,
    );
    let start_angle = (t1.1 - center.1).atan2(t1.0 - center.0);
    let mut end_angle = (t2.1 - center.1).atan2(t2.0 - center.0);
    let sweep_clockwise = cross < 0.0;
    if sweep_clockwise {
        while end_angle <= start_angle {
            end_angle += std::f64::consts::TAU;
        }
    } else {
        while end_angle >= start_angle {
            end_angle -= std::f64::consts::TAU;
        }
    }
    let sweep = end_angle - start_angle;
    let segments = ((sweep.abs() / (std::f64::consts::FRAC_PI_2)).ceil() as u32).max(1);
    let step = sweep / f64::from(segments);
    let kappa = 4.0 / 3.0 * (step / 4.0).tan();
    let mut current = start_angle;
    for _ in 0..segments {
        let next = current + step;
        let p0 = (
            center.0 + radius * current.cos(),
            center.1 + radius * current.sin(),
        );
        let p3 = (center.0 + radius * next.cos(), center.1 + radius * next.sin());
        let d0 = (-current.sin(), current.cos());
        let d1 = (-next.sin(), next.cos());
        let p1 = (p0.0 + kappa * radius * d0.0, p0.1 + kappa * radius * d0.1);
        let p2 = (p3.0 - kappa * radius * d1.0, p3.1 - kappa * radius * d1.1);
        builder.cubic_to(
            p1.0 as f32,
            p1.1 as f32,
            p2.0 as f32,
            p2.1 as f32,
            p3.0 as f32,
            p3.1 as f32,
        );
        current = next;
    }
}

pub static RECTANGLE_GRAPHIC_DEFINITION: GraphicDefinition = GraphicDefinition {
    id: "rectangle",
    name: "Rectangle",
    keywords: &["rectangle", "square", "box"],
    params: &[
        FILL_PARAM,
        STROKE_PARAM,
        STROKE_WIDTH_PARAM,
        STROKE_ALIGN_PARAM,
        CORNER_RADIUS_PARAM,
    ],
    render: render_rectangle,
};

fn render_rectangle(context: &mut crate::types::GraphicRenderContext) -> Result<(), GraphicsError> {
    let params = resolve_shape_params(context.params)?;
    let width = f64::from(context.width);
    let height = f64::from(context.height);
    let draw_width = (width - params.inset * 2.0).max(1.0);
    let draw_height = (height - params.inset * 2.0).max(1.0);
    let radius_percent = param_number(context.params, "cornerRadius", 0.0).max(0.0);
    let radius = (draw_width.min(draw_height) / 2.0) * radius_percent.min(50.0) / 50.0;
    let mut builder = PathBuilder::new();
    push_round_rect(&mut builder, params.inset, params.inset, draw_width, draw_height, radius);
    let Some(path) = builder.finish() else {
        return Ok(());
    };
    fill_and_stroke(context.pixmap, &path, &params);
    Ok(())
}

pub static ELLIPSE_GRAPHIC_DEFINITION: GraphicDefinition = GraphicDefinition {
    id: "ellipse",
    name: "Ellipse",
    keywords: &["ellipse", "circle", "oval"],
    params: &[
        FILL_PARAM,
        STROKE_PARAM,
        STROKE_WIDTH_PARAM,
        STROKE_ALIGN_PARAM,
    ],
    render: render_ellipse,
};

fn render_ellipse(context: &mut crate::types::GraphicRenderContext) -> Result<(), GraphicsError> {
    let params = resolve_shape_params(context.params)?;
    let width = f64::from(context.width);
    let height = f64::from(context.height);
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let radius_x = (width / 2.0 - params.inset).max(1.0);
    let radius_y = (height / 2.0 - params.inset).max(1.0);
    let mut builder = PathBuilder::new();
    if let Some(rect) = Rect::from_xywh(
        (center_x - radius_x) as f32,
        (center_y - radius_y) as f32,
        (radius_x * 2.0) as f32,
        (radius_y * 2.0) as f32,
    ) {
        builder.push_oval(rect);
    }
    let Some(path) = builder.finish() else {
        return Ok(());
    };
    fill_and_stroke(context.pixmap, &path, &params);
    Ok(())
}

const SIDES_PARAM: ParamDefinition = ParamDefinition::Number {
    key: "sides",
    label: "Sides",
    group: None,
    default: 5.0,
    min: 3.0,
    max: Some(12.0),
    step: 1.0,
    short_label: Some("S"),
};

pub static POLYGON_GRAPHIC_DEFINITION: GraphicDefinition = GraphicDefinition {
    id: "polygon",
    name: "Polygon",
    keywords: &["polygon", "triangle", "pentagon", "hexagon", "diamond"],
    params: &[
        FILL_PARAM,
        STROKE_PARAM,
        STROKE_WIDTH_PARAM,
        STROKE_ALIGN_PARAM,
        SIDES_PARAM,
        CORNER_RADIUS_PARAM,
    ],
    render: render_polygon,
};

fn build_polygon_vertices(
    center_x: f64,
    center_y: f64,
    radius: f64,
    sides: usize,
) -> Vec<Point> {
    (0..sides)
        .map(|index| {
            let angle = -std::f64::consts::FRAC_PI_2
                + (index as f64 * std::f64::consts::TAU) / sides as f64;
            Point::from_xy(
                (center_x + angle.cos() * radius) as f32,
                (center_y + angle.sin() * radius) as f32,
            )
        })
        .collect()
}

fn trace_rounded_polygon_path(
    builder: &mut PathBuilder,
    vertices: &[Point],
    radius: f64,
) {
    if vertices.len() < 3 {
        return;
    }
    if radius <= 0.0 {
        builder.move_to(vertices[0].x, vertices[0].y);
        for vertex in &vertices[1..] {
            builder.line_to(vertex.x, vertex.y);
        }
        builder.close();
        return;
    }
    let count = vertices.len();
    for index in 0..count {
        let previous = vertices[(index + count - 1) % count];
        let current = vertices[index];
        let next = vertices[(index + 1) % count];
        let to_previous = normalize(
            f64::from(previous.x - current.x),
            f64::from(previous.y - current.y),
        );
        let to_next = normalize(f64::from(next.x - current.x), f64::from(next.y - current.y));
        let angle = (to_previous.0 * to_next.0 + to_previous.1 * to_next.1)
            .clamp(-1.0, 1.0)
            .acos();
        let dist_prev = (f64::from(previous.x - current.x)).hypot(f64::from(previous.y - current.y));
        let dist_next = (f64::from(next.x - current.x)).hypot(f64::from(next.y - current.y));
        let max_offset = dist_prev.min(dist_next) / 2.0;
        let tangent_offset = (radius / (angle / 2.0).tan()).min(max_offset);
        let start = (
            f64::from(current.x) + to_previous.0 * tangent_offset,
            f64::from(current.y) + to_previous.1 * tangent_offset,
        );
        let end = (
            f64::from(current.x) + to_next.0 * tangent_offset,
            f64::from(current.y) + to_next.1 * tangent_offset,
        );
        if index == 0 {
            builder.move_to(start.0 as f32, start.1 as f32);
        } else {
            builder.line_to(start.0 as f32, start.1 as f32);
        }
        arc_to(
            builder,
            f64::from(current.x),
            f64::from(current.y),
            end.0,
            end.1,
            radius.min(max_offset),
        );
    }
    builder.close();
}

fn normalize(x: f64, y: f64) -> (f64, f64) {
    let length = x.hypot(y);
    if length == 0.0 {
        return (0.0, 0.0);
    }
    (x / length, y / length)
}

fn render_polygon(context: &mut crate::types::GraphicRenderContext) -> Result<(), GraphicsError> {
    let params = resolve_shape_params(context.params)?;
    let width = f64::from(context.width);
    let height = f64::from(context.height);
    let sides = (param_number(context.params, "sides", 5.0).round() as i64).clamp(3, 12) as usize;
    let radius = (width.min(height) / 2.0 - params.inset).max(1.0);
    let max_corner_radius = radius * (std::f64::consts::PI / sides as f64).sin();
    let corner_radius_percent = param_number(context.params, "cornerRadius", 0.0).max(0.0);
    let corner_radius = max_corner_radius * corner_radius_percent.min(50.0) / 50.0;
    let vertices = build_polygon_vertices(width / 2.0, height / 2.0, radius, sides);
    let mut builder = PathBuilder::new();
    trace_rounded_polygon_path(&mut builder, &vertices, corner_radius);
    let Some(path) = builder.finish() else {
        return Ok(());
    };
    fill_and_stroke(context.pixmap, &path, &params);
    Ok(())
}

const POINTS_PARAM: ParamDefinition = ParamDefinition::Number {
    key: "points",
    label: "Points",
    group: None,
    default: 5.0,
    min: 3.0,
    max: Some(12.0),
    step: 1.0,
    short_label: Some("P"),
};

const DEPTH_PARAM: ParamDefinition = ParamDefinition::Number {
    key: "depth",
    label: "Depth",
    group: None,
    default: 45.0,
    min: 1.0,
    max: Some(99.0),
    step: 1.0,
    short_label: Some("D"),
};

pub static STAR_GRAPHIC_DEFINITION: GraphicDefinition = GraphicDefinition {
    id: "star",
    name: "Star",
    keywords: &["star", "sparkle", "burst"],
    params: &[
        FILL_PARAM,
        STROKE_PARAM,
        STROKE_WIDTH_PARAM,
        STROKE_ALIGN_PARAM,
        POINTS_PARAM,
        DEPTH_PARAM,
    ],
    render: render_star,
};

fn render_star(context: &mut crate::types::GraphicRenderContext) -> Result<(), GraphicsError> {
    let params = resolve_shape_params(context.params)?;
    let width = f64::from(context.width);
    let height = f64::from(context.height);
    let points = (param_number(context.params, "points", 5.0).round() as i64).clamp(3, 12) as usize;
    let depth = (param_number(context.params, "depth", 45.0).clamp(1.0, 99.0)) / 100.0;
    let outer_radius = (width.min(height) / 2.0 - params.inset).max(1.0);
    let inner_radius = outer_radius * depth;
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let mut builder = PathBuilder::new();
    for index in 0..points * 2 {
        let radius = if index % 2 == 0 {
            outer_radius
        } else {
            inner_radius
        };
        let angle = -std::f64::consts::FRAC_PI_2 + (index as f64 * std::f64::consts::PI) / points as f64;
        let x = center_x + angle.cos() * radius;
        let y = center_y + angle.sin() * radius;
        if index == 0 {
            builder.move_to(x as f32, y as f32);
        } else {
            builder.line_to(x as f32, y as f32);
        }
    }
    builder.close();
    let Some(path) = builder.finish() else {
        return Ok(());
    };
    fill_and_stroke(context.pixmap, &path, &params);
    Ok(())
}

pub fn default_graphic_definitions() -> Vec<&'static GraphicDefinition> {
    vec![
        &RECTANGLE_GRAPHIC_DEFINITION,
        &ELLIPSE_GRAPHIC_DEFINITION,
        &POLYGON_GRAPHIC_DEFINITION,
        &STAR_GRAPHIC_DEFINITION,
    ]
}
