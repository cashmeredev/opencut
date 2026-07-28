use tiny_skia::{BlendMode, Color, FillRule, Paint, Path, Pixmap, PixmapPaint, Stroke, Transform};

use crate::definitions::StrokeAlign;

pub fn apply_aligned_stroke(
    pixmap: &mut Pixmap,
    path: &Path,
    stroke_width: f64,
    stroke_align: StrokeAlign,
    stroke_color: Color,
) {
    if stroke_width <= 0.0 {
        return;
    }

    match stroke_align {
        StrokeAlign::Inside => {
            let Some(mut stroke_pixmap) = Pixmap::new(pixmap.width(), pixmap.height()) else {
                return;
            };
            stroke_path(&mut stroke_pixmap, path, stroke_width * 2.0, stroke_color);
            erase_outside(&mut stroke_pixmap, path, BlendMode::DestinationIn);
            composite(pixmap, &stroke_pixmap);
        }
        StrokeAlign::Outside => {
            let Some(mut stroke_pixmap) = Pixmap::new(pixmap.width(), pixmap.height()) else {
                return;
            };
            stroke_path(&mut stroke_pixmap, path, stroke_width * 2.0, stroke_color);
            erase_outside(&mut stroke_pixmap, path, BlendMode::DestinationOut);
            composite(pixmap, &stroke_pixmap);
        }
        StrokeAlign::Center => {
            stroke_path(pixmap, path, stroke_width, stroke_color);
        }
    }
}

fn stroke_path(pixmap: &mut Pixmap, path: &Path, width: f64, color: Color) {
    let paint = Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        anti_alias: true,
        ..Paint::default()
    };
    let stroke = Stroke {
        width: width as f32,
        ..Stroke::default()
    };
    pixmap.stroke_path(path, &paint, &stroke, Transform::default(), None);
}

fn erase_outside(pixmap: &mut Pixmap, path: &Path, blend_mode: BlendMode) {
    let paint = Paint {
        shader: tiny_skia::Shader::SolidColor(Color::BLACK),
        anti_alias: true,
        blend_mode,
        ..Paint::default()
    };
    pixmap.fill_path(path, &paint, FillRule::Winding, Transform::default(), None);
}

fn composite(target: &mut Pixmap, source: &Pixmap) {
    target.draw_pixmap(
        0,
        0,
        source.as_ref(),
        &PixmapPaint::default(),
        Transform::default(),
        None,
    );
}
