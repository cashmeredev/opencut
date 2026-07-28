use scene::StrokeAlign;
use tiny_skia::{Color, Paint, Path, Pixmap, PremultipliedColorU8, Stroke, Transform};

use crate::MaskRenderError;

pub fn white_paint() -> Paint<'static> {
    Paint {
        shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 255, 255, 255)),
        anti_alias: true,
        ..Paint::default()
    }
}

pub fn fill_white(path: &Option<Path>, pixmap: &mut Pixmap) {
    if let Some(path) = path {
        pixmap.fill_path(
            path,
            &white_paint(),
            tiny_skia::FillRule::Winding,
            Transform::default(),
            None,
        );
    }
}

pub fn parse_stroke_color(value: &str) -> Result<Color, MaskRenderError> {
    let hex = value
        .trim()
        .strip_prefix('#')
        .ok_or_else(|| MaskRenderError::InvalidStrokeColor(value.to_string()))?;
    let expand = |c: u8| (c << 4) | c;
    let (r, g, b, a) = match hex.len() {
        3 => {
            let v = u32::from_str_radix(hex, 16)
                .map_err(|_| MaskRenderError::InvalidStrokeColor(value.to_string()))?;
            (
                expand(((v >> 8) & 0xF) as u8),
                expand(((v >> 4) & 0xF) as u8),
                expand((v & 0xF) as u8),
                255,
            )
        }
        4 => {
            let v = u32::from_str_radix(hex, 16)
                .map_err(|_| MaskRenderError::InvalidStrokeColor(value.to_string()))?;
            (
                expand(((v >> 12) & 0xF) as u8),
                expand(((v >> 8) & 0xF) as u8),
                expand(((v >> 4) & 0xF) as u8),
                expand((v & 0xF) as u8),
            )
        }
        6 => {
            let v = u32::from_str_radix(hex, 16)
                .map_err(|_| MaskRenderError::InvalidStrokeColor(value.to_string()))?;
            (((v >> 16) & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8, 255)
        }
        8 => {
            let v = u32::from_str_radix(hex, 16)
                .map_err(|_| MaskRenderError::InvalidStrokeColor(value.to_string()))?;
            (
                ((v >> 24) & 0xFF) as u8,
                ((v >> 16) & 0xFF) as u8,
                ((v >> 8) & 0xFF) as u8,
                (v & 0xFF) as u8,
            )
        }
        _ => return Err(MaskRenderError::InvalidStrokeColor(value.to_string())),
    };
    Ok(Color::from_rgba8(r, g, b, a))
}

pub fn canvas_stroke(stroke_width: f64) -> Stroke {
    Stroke {
        width: stroke_width as f32,
        miter_limit: 10.0,
        line_cap: tiny_skia::LineCap::Butt,
        line_join: tiny_skia::LineJoin::Miter,
        ..Stroke::default()
    }
}

pub fn round_stroke(stroke_width: f64) -> Stroke {
    Stroke {
        width: stroke_width as f32,
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    }
}

pub fn stroke_path_with(
    path: &Option<Path>,
    stroke: &Stroke,
    color: Color,
    pixmap: &mut Pixmap,
) {
    if let Some(path) = path {
        let paint = Paint {
            shader: tiny_skia::Shader::SolidColor(color),
            anti_alias: true,
            ..Paint::default()
        };
        pixmap.stroke_path(path, &paint, stroke, Transform::default(), None);
    }
}

pub fn apply_stroke_align(stroke: &mut Pixmap, fill: &Pixmap, align: StrokeAlign) {
    let keep_inside = match align {
        StrokeAlign::Center => return,
        StrokeAlign::Inside => true,
        StrokeAlign::Outside => false,
    };
    for (dst, src) in stroke.pixels_mut().iter_mut().zip(fill.pixels().iter()) {
        let coverage = if keep_inside {
            src.alpha() as u16
        } else {
            255 - src.alpha() as u16
        };
        let scale = |channel: u8| ((channel as u16 * coverage + 127) / 255) as u8;
        let scaled = PremultipliedColorU8::from_rgba(
            scale(dst.red()),
            scale(dst.green()),
            scale(dst.blue()),
            scale(dst.alpha()),
        );
        if let Some(scaled) = scaled {
            *dst = scaled;
        } else {
            *dst = PremultipliedColorU8::from_rgba(0, 0, 0, 0).unwrap();
        }
    }
}
