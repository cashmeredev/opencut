use tiny_skia::{
    Color as SkColor, GradientStop, LinearGradient, Paint, Pixmap, Point, RadialGradient, Rect,
    Shader, SpreadMode, Transform,
};

use super::ast::{
    Distance, GradientAst, GradientOrientation, LinearOrientation, Position, RadialOrientation,
    ShapeKind, ShapeStyle,
};
use super::parser::parse_gradient;
use super::stops::normalize_color_stops;
use crate::color::{Rgba, parse_css_color};
use crate::error::GraphicsError;

#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundLayer {
    Color(String),
    Gradient(GradientAst),
}

pub fn is_gradient_layer(segment: &str) -> bool {
    let names = [
        "linear-gradient",
        "repeating-linear-gradient",
        "radial-gradient",
        "repeating-radial-gradient",
    ];
    let mut offset = 0;
    let mut rest = segment;
    if rest.starts_with('-') {
        for vendor in ["-webkit-", "-o-", "-ms-", "-moz-"] {
            if rest.len() >= vendor.len() && rest[..vendor.len()].eq_ignore_ascii_case(vendor) {
                offset = vendor.len();
                break;
            }
        }
        if offset == 0 {
            return false;
        }
        rest = &rest[offset..];
    }
    names
        .iter()
        .any(|name| rest.len() >= name.len() && rest[..name.len()].eq_ignore_ascii_case(name))
}

pub fn split_css_layers(css: &str) -> Vec<String> {
    let mut layers = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    for ch in css.chars() {
        if ch == '(' {
            depth += 1;
        }
        if ch == ')' {
            depth = depth.saturating_sub(1);
        }
        if ch == ',' && depth == 0 {
            layers.push(current.trim().to_string());
            current = String::new();
            continue;
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        layers.push(current.trim().to_string());
    }
    layers
}

pub fn parse_background_layers(css: &str) -> Vec<BackgroundLayer> {
    let mut layers = Vec::new();
    for segment in split_css_layers(css) {
        if segment.is_empty() {
            continue;
        }
        if is_gradient_layer(&segment) {
            match parse_gradient(segment.trim()) {
                Ok(parsed) => {
                    for gradient in parsed {
                        layers.push(BackgroundLayer::Gradient(gradient));
                    }
                    continue;
                }
                Err(_) => {
                    layers.push(BackgroundLayer::Color(segment));
                    continue;
                }
            }
        }
        layers.push(BackgroundLayer::Color(segment));
    }
    layers
}

pub fn draw_css_background(pixmap: &mut Pixmap, css: &str) -> Result<(), GraphicsError> {
    let width = f64::from(pixmap.width());
    let height = f64::from(pixmap.height());
    let layers = parse_background_layers(css);
    for layer in layers.iter().rev() {
        match layer {
            BackgroundLayer::Color(value) => {
                let color = parse_css_color(value)
                    .ok_or_else(|| GraphicsError::InvalidColor(value.clone()))?;
                fill_solid(pixmap, color_to_sk(color));
            }
            BackgroundLayer::Gradient(gradient) => {
                draw_gradient_layer(pixmap, width, height, gradient)?;
            }
        }
    }
    Ok(())
}

fn fill_solid(pixmap: &mut Pixmap, color: SkColor) {
    let Some(rect) = Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
    else {
        return;
    };
    let paint = Paint {
        shader: Shader::SolidColor(color),
        ..Paint::default()
    };
    pixmap.fill_rect(rect, &paint, Transform::default(), None);
}

fn color_to_sk(color: Rgba) -> SkColor {
    SkColor::from_rgba(
        color.r.clamp(0.0, 1.0) as f32,
        color.g.clamp(0.0, 1.0) as f32,
        color.b.clamp(0.0, 1.0) as f32,
        color.a.clamp(0.0, 1.0) as f32,
    )
    .unwrap_or(SkColor::TRANSPARENT)
}

fn build_stops(
    gradient: &GradientAst,
    gradient_length: f64,
) -> Result<Vec<GradientStop>, GraphicsError> {
    let normalized = normalize_color_stops(&gradient.color_stops, gradient_length);
    let mut stops = Vec::with_capacity(normalized.len());
    for stop in normalized {
        let color = parse_css_color(&stop.color)
            .ok_or_else(|| GraphicsError::InvalidColor(stop.color.clone()))?;
        stops.push(GradientStop::new(stop.offset as f32, color_to_sk(color)));
    }
    Ok(stops)
}

fn full_rect(width: f64, height: f64) -> Option<Rect> {
    Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
}

fn draw_gradient_layer(
    pixmap: &mut Pixmap,
    width: f64,
    height: f64,
    gradient: &GradientAst,
) -> Result<(), GraphicsError> {
    if gradient.kind.is_linear() {
        draw_linear_gradient(pixmap, width, height, gradient)
    } else {
        draw_radial_gradient(pixmap, width, height, gradient)
    }
}

struct LinearPoints {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    length: f64,
}

fn resolve_linear_points(
    width: f64,
    height: f64,
    orientation: Option<&GradientOrientation>,
) -> LinearPoints {
    let angle = resolve_linear_angle(orientation);
    let radians = angle.to_radians();
    let dx = radians.sin();
    let dy = -radians.cos();
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let half_length = ((width * dx).abs() + (height * dy).abs()) / 2.0;
    let x0 = center_x - dx * half_length;
    let y0 = center_y - dy * half_length;
    let x1 = center_x + dx * half_length;
    let y1 = center_y + dy * half_length;
    let length = (x1 - x0).hypot(y1 - y0);
    LinearPoints { x0, y0, x1, y1, length }
}

fn resolve_linear_angle(orientation: Option<&GradientOrientation>) -> f64 {
    let Some(orientation) = orientation else {
        return 180.0;
    };
    if let GradientOrientation::Linear(linear) = orientation {
        match linear {
            LinearOrientation::Angular(value) => {
                return value.parse::<f64>().unwrap_or(180.0);
            }
            LinearOrientation::Directional(value) => {
                return angle_from_directional(value);
            }
        }
    }
    180.0
}

fn angle_from_directional(value: &str) -> f64 {
    let normalized = value.to_lowercase().replace("to", "");
    let mut dx: f64 = 0.0;
    let mut dy: f64 = 0.0;
    for part in normalized.split_whitespace() {
        match part {
            "left" => dx = -1.0,
            "right" => dx = 1.0,
            "top" => dy = -1.0,
            "bottom" => dy = 1.0,
            _ => {}
        }
    }
    if dx == 0.0 && dy == 0.0 {
        return 180.0;
    }
    let angle = dx.atan2(-dy).to_degrees();
    (angle + 360.0) % 360.0
}

fn draw_linear_gradient(
    pixmap: &mut Pixmap,
    width: f64,
    height: f64,
    gradient: &GradientAst,
) -> Result<(), GraphicsError> {
    let points = resolve_linear_points(width, height, gradient.orientation.as_ref());
    let stops = build_stops(gradient, points.length)?;
    let (Some(shader), Some(rect)) = (
        LinearGradient::new(
            Point::from_xy(points.x0 as f32, points.y0 as f32),
            Point::from_xy(points.x1 as f32, points.y1 as f32),
            stops,
            SpreadMode::Pad,
            Transform::default(),
        ),
        full_rect(width, height),
    ) else {
        return Ok(());
    };
    let paint = Paint {
        shader,
        ..Paint::default()
    };
    pixmap.fill_rect(rect, &paint, Transform::default(), None);
    Ok(())
}

struct RadialDimensions {
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
}

fn resolve_radial_dimensions(
    width: f64,
    height: f64,
    orientation: Option<&GradientOrientation>,
) -> RadialDimensions {
    let center_fallback = (width / 2.0, height / 2.0);
    let radial = match orientation {
        Some(GradientOrientation::Radial(list)) => list.first(),
        _ => None,
    };

    let Some(radial) = radial else {
        let (rx, ry) = resolve_radial_extents(
            width,
            height,
            center_fallback.0,
            center_fallback.1,
            ShapeKind::Ellipse,
            "farthest-corner",
        );
        return RadialDimensions {
            cx: center_fallback.0,
            cy: center_fallback.1,
            rx,
            ry,
        };
    };

    match radial {
        RadialOrientation::Shape(shape) => {
            let (cx, cy) = resolve_radial_center(width, height, shape.at.as_ref());
            if let Some(ShapeStyle::Position(position)) = &shape.style {
                let rx = position
                    .x
                    .as_ref()
                    .map(|d| resolve_ellipse_dimension(d, width))
                    .unwrap_or(width / 2.0);
                let ry = position
                    .y
                    .as_ref()
                    .map(|d| resolve_ellipse_dimension(d, height))
                    .unwrap_or(height / 2.0);
                return RadialDimensions { cx, cy, rx, ry };
            }
            if let Some(ShapeStyle::Distance(distance)) = &shape.style {
                let resolved =
                    resolve_distance_in_pixels(distance, width.max(height)).unwrap_or(0.0);
                return RadialDimensions {
                    cx,
                    cy,
                    rx: resolved,
                    ry: resolved,
                };
            }
            let extent = match &shape.style {
                Some(ShapeStyle::ExtentKeyword(value)) => value.as_str(),
                _ => "farthest-corner",
            };
            let (rx, ry) = resolve_radial_extents(width, height, cx, cy, shape.value, extent);
            RadialDimensions { cx, cy, rx, ry }
        }
        RadialOrientation::ExtentKeyword { value, at } => {
            let (cx, cy) = resolve_radial_center(width, height, at.as_ref());
            let (rx, ry) =
                resolve_radial_extents(width, height, cx, cy, ShapeKind::Ellipse, value);
            RadialDimensions { cx, cy, rx, ry }
        }
        RadialOrientation::DefaultRadial { at } => {
            let (cx, cy) = resolve_radial_center(width, height, Some(at));
            let (rx, ry) = resolve_radial_extents(
                width,
                height,
                cx,
                cy,
                ShapeKind::Ellipse,
                "farthest-corner",
            );
            RadialDimensions { cx, cy, rx, ry }
        }
    }
}

fn resolve_radial_center(width: f64, height: f64, position: Option<&Position>) -> (f64, f64) {
    let Some(position) = position else {
        return (width / 2.0, height / 2.0);
    };
    let normalized = normalize_position_keywords(position);
    let cx = resolve_position_value(normalized.x.as_ref(), width, true);
    let cy = resolve_position_value(normalized.y.as_ref(), height, false);
    (cx, cy)
}

fn normalize_position_keywords(position: &Position) -> Position {
    if let (Some(Distance::PositionKeyword(x_kw)), Some(Distance::PositionKeyword(y_kw))) =
        (&position.x, &position.y)
    {
        let x_keyword = x_kw.to_lowercase();
        let y_keyword = y_kw.to_lowercase();
        let x_is_vertical = x_keyword == "top" || x_keyword == "bottom";
        let y_is_horizontal = y_keyword == "left" || y_keyword == "right";
        if x_is_vertical && y_is_horizontal {
            return Position {
                x: Some(Distance::PositionKeyword(y_keyword)),
                y: Some(Distance::PositionKeyword(x_keyword)),
            };
        }
    }
    position.clone()
}

fn resolve_radial_extents(
    width: f64,
    height: f64,
    cx: f64,
    cy: f64,
    shape: ShapeKind,
    extent: &str,
) -> (f64, f64) {
    let left = cx;
    let right = width - cx;
    let top = cy;
    let bottom = height - cy;

    if shape == ShapeKind::Circle {
        let distances = [
            left.hypot(top),
            right.hypot(top),
            left.hypot(bottom),
            right.hypot(bottom),
        ];
        return match extent {
            "closest-side" => (left.min(right).min(top).min(bottom), 0.0),
            "farthest-side" => (left.max(right).max(top).max(bottom), 0.0),
            "closest-corner" => (distances.iter().cloned().fold(f64::INFINITY, f64::min), 0.0),
            _ => (distances.iter().cloned().fold(f64::NEG_INFINITY, f64::max), 0.0),
        };
    }

    match extent {
        "closest-side" => (left.min(right), top.min(bottom)),
        "farthest-side" => (left.max(right), top.max(bottom)),
        _ => {
            let corners = [(left, top), (right, top), (left, bottom), (right, bottom)];
            let chosen = if extent == "closest-corner" {
                corners
                    .iter()
                    .min_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)))
            } else {
                corners
                    .iter()
                    .max_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)))
            };
            let chosen = chosen.copied().unwrap_or((width / 2.0, height / 2.0));
            (chosen.0.abs(), chosen.1.abs())
        }
    }
}

fn resolve_position_value(distance: Option<&Distance>, axis_size: f64, is_x: bool) -> f64 {
    let Some(distance) = distance else {
        return axis_size / 2.0;
    };
    match distance {
        Distance::Percent(value) => value
            .parse::<f64>()
            .map(|v| (v / 100.0) * axis_size)
            .unwrap_or(axis_size / 2.0),
        Distance::PositionKeyword(value) => keyword_to_position(value, axis_size, is_x),
        Distance::Px(value) => value.parse::<f64>().unwrap_or(axis_size / 2.0),
        Distance::Em(value) => value
            .parse::<f64>()
            .map(|v| v * 16.0)
            .unwrap_or(axis_size / 2.0),
        _ => axis_size / 2.0,
    }
}

fn keyword_to_position(value: &str, axis_size: f64, is_x: bool) -> f64 {
    let keyword = value.to_lowercase();
    if keyword == "center" {
        return axis_size / 2.0;
    }
    if is_x {
        if keyword == "left" {
            return 0.0;
        }
        if keyword == "right" {
            return axis_size;
        }
    } else {
        if keyword == "top" {
            return 0.0;
        }
        if keyword == "bottom" {
            return axis_size;
        }
    }
    axis_size / 2.0
}

fn resolve_distance_in_pixels(distance: &Distance, axis_size: f64) -> Option<f64> {
    match distance {
        Distance::Percent(value) => value.parse::<f64>().ok().map(|v| (v / 100.0) * axis_size),
        Distance::Px(value) => value.parse::<f64>().ok(),
        Distance::Em(value) => value.parse::<f64>().ok().map(|v| v * 16.0),
        _ => None,
    }
}

fn resolve_ellipse_dimension(distance: &Distance, axis_size: f64) -> f64 {
    match distance {
        Distance::Percent(value) => value
            .parse::<f64>()
            .map(|v| (v / 100.0) * axis_size)
            .unwrap_or(axis_size / 2.0),
        Distance::Px(value) => value.parse::<f64>().unwrap_or(axis_size / 2.0),
        Distance::Em(value) => value
            .parse::<f64>()
            .map(|v| v * 16.0)
            .unwrap_or(axis_size / 2.0),
        _ => axis_size / 2.0,
    }
}

fn draw_radial_gradient(
    pixmap: &mut Pixmap,
    width: f64,
    height: f64,
    gradient: &GradientAst,
) -> Result<(), GraphicsError> {
    let dims = resolve_radial_dimensions(width, height, gradient.orientation.as_ref());
    let gradient_length = dims.rx.max(dims.ry);
    let stops = build_stops(gradient, gradient_length)?;

    if dims.rx == dims.ry || dims.ry == 0.0 {
        let Some(shader) = RadialGradient::new(
            Point::from_xy(dims.cx as f32, dims.cy as f32),
            Point::from_xy(dims.cx as f32, dims.cy as f32),
            dims.rx as f32,
            stops,
            SpreadMode::Pad,
            Transform::default(),
        ) else {
            return Ok(());
        };
        if let Some(rect) = full_rect(width, height) {
            let paint = Paint {
                shader,
                ..Paint::default()
            };
            pixmap.fill_rect(rect, &paint, Transform::default(), None);
        }
        return Ok(());
    }

    if dims.rx <= 0.0 {
        return Ok(());
    }
    let scale_y = dims.ry / dims.rx;
    if !scale_y.is_finite() || scale_y <= 0.0 {
        return Ok(());
    }
    let transform = Transform::from_row(1.0, 0.0, 0.0, scale_y as f32, dims.cx as f32, dims.cy as f32);
    let Some(shader) = RadialGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(0.0, 0.0),
        dims.rx as f32,
        stops,
        SpreadMode::Pad,
        transform,
    ) else {
        return Ok(());
    };
    if let Some(rect) = full_rect(width, height) {
        let paint = Paint {
            shader,
            ..Paint::default()
        };
        pixmap.fill_rect(rect, &paint, Transform::default(), None);
    }
    Ok(())
}
