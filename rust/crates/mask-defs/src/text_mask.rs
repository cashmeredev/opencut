use std::cell::RefCell;

use cosmic_text::{
    Attrs, Buffer, CacheKey, CacheKeyFlags, Command, Family, FontSystem, Metrics, Shaping,
    Style, SwashCache, Weight, Wrap, fontdb,
};
use scene::{TextDecoration, TextFontStyle, TextFontWeight, TextMaskParams};
use tiny_skia::{PathBuilder, Pixmap};

use crate::MaskRenderError;
use crate::paint;

const FONT_SIZE_SCALE_REFERENCE: f64 = 90.0;
const TEXT_DECORATION_THICKNESS_RATIO: f64 = 0.07;
const STRIKETHROUGH_VERTICAL_RATIO: f64 = 0.35;

thread_local! {
    static TEXT_ENGINE: RefCell<(FontSystem, SwashCache)> = RefCell::new({
        let mut font_system = FontSystem::new();
        let nixos_fonts = std::path::Path::new("/run/current-system/sw/share/X11/fonts");
        if nixos_fonts.is_dir() {
            font_system.db_mut().load_fonts_dir(nixos_fonts);
        }
        (font_system, SwashCache::new())
    });
}

struct PositionedGlyph {
    font_id: fontdb::ID,
    glyph_id: u16,
    font_size: f32,
    font_weight: fontdb::Weight,
    flags: CacheKeyFlags,
    x: f32,
    baseline: f32,
}

struct LineMetrics {
    y_center: f32,
    width: f32,
    ascent: f32,
    descent: f32,
}

struct TextLayout {
    glyphs: Vec<PositionedGlyph>,
    lines: Vec<LineMetrics>,
}

#[derive(Clone, Copy)]
struct TextTransform {
    cos: f32,
    sin: f32,
    scale: f32,
    translate_x: f32,
    translate_y: f32,
}

impl TextTransform {
    fn new(params: &TextMaskParams, width: f64, height: f64) -> Self {
        let rotation_rad = params.rotation.to_radians();
        TextTransform {
            cos: rotation_rad.cos() as f32,
            sin: rotation_rad.sin() as f32,
            scale: params.scale as f32,
            translate_x: (width / 2.0 + params.center_x * width) as f32,
            translate_y: (height / 2.0 + params.center_y * height) as f32,
        }
    }

    fn map(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.translate_x + self.scale * (x * self.cos - y * self.sin),
            self.translate_y + self.scale * (x * self.sin + y * self.cos),
        )
    }
}

fn layout_text(
    font_system: &mut FontSystem,
    params: &TextMaskParams,
    canvas_height: f64,
) -> TextLayout {
    let empty = TextLayout { glyphs: Vec::new(), lines: Vec::new() };
    let scaled_font_size = params.font_size * (canvas_height / FONT_SIZE_SCALE_REFERENCE);
    if !scaled_font_size.is_finite() || scaled_font_size <= 0.0 {
        return empty;
    }
    let mut line_height_px = scaled_font_size * params.line_height;
    if !line_height_px.is_finite() || line_height_px <= 0.0 {
        line_height_px = scaled_font_size;
    }

    let weight = match params.font_weight {
        TextFontWeight::Normal => Weight::NORMAL,
        TextFontWeight::Bold => Weight::BOLD,
    };
    let style = match params.font_style {
        TextFontStyle::Normal => Style::Normal,
        TextFontStyle::Italic => Style::Italic,
    };
    let attrs = Attrs::new()
        .family(Family::Name(&params.font_family))
        .weight(weight)
        .style(style)
        .letter_spacing((params.letter_spacing / scaled_font_size) as f32);

    let content_lines: Vec<&str> = params.content.split('\n').collect();
    let visual_center_offset =
        (content_lines.len().saturating_sub(1)) as f64 * line_height_px / 2.0;

    let mut layout = TextLayout { glyphs: Vec::new(), lines: Vec::new() };
    for (index, line) in content_lines.iter().enumerate() {
        let y_center = (index as f64 * line_height_px - visual_center_offset) as f32;
        if line.is_empty() {
            layout.lines.push(LineMetrics { y_center, width: 0.0, ascent: 0.0, descent: 0.0 });
            continue;
        }

        let mut buffer = Buffer::new(
            font_system,
            Metrics::new(scaled_font_size as f32, line_height_px as f32),
        );
        buffer.set_wrap(font_system, Wrap::None);
        buffer.set_size(font_system, None, None);
        buffer.set_text(font_system, line, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);

        let mut borrowed = buffer.borrow_with(font_system);
        let Some(layout_lines) = borrowed.line_layout(0) else {
            continue;
        };
        let Some(layout_line) = layout_lines.first() else {
            continue;
        };

        let baseline = y_center + (layout_line.max_ascent - layout_line.max_descent) / 2.0;
        for glyph in &layout_line.glyphs {
            layout.glyphs.push(PositionedGlyph {
                font_id: glyph.font_id,
                glyph_id: glyph.glyph_id,
                font_size: glyph.font_size,
                font_weight: glyph.font_weight,
                flags: glyph.cache_key_flags,
                x: -layout_line.w / 2.0 + glyph.x + glyph.x_offset * glyph.font_size,
                baseline: baseline + glyph.y - glyph.y_offset * glyph.font_size,
            });
        }
        layout.lines.push(LineMetrics {
            y_center,
            width: layout_line.w,
            ascent: layout_line.max_ascent,
            descent: layout_line.max_descent,
        });
    }
    layout
}

fn push_glyph_outlines(
    font_system: &mut FontSystem,
    cache: &mut SwashCache,
    layout: &TextLayout,
    transform: &TextTransform,
    builder: &mut PathBuilder,
) {
    for glyph in &layout.glyphs {
        let (cache_key, _, _) = CacheKey::new(
            glyph.font_id,
            glyph.glyph_id,
            glyph.font_size,
            (0.0, 0.0),
            glyph.font_weight,
            glyph.flags,
        );
        let Some(commands) = cache.get_outline_commands(font_system, cache_key) else {
            continue;
        };
        for command in commands {
            match *command {
                Command::MoveTo(p) => {
                    let (x, y) = transform.map(glyph.x + p.x, glyph.baseline - p.y);
                    builder.move_to(x, y);
                }
                Command::LineTo(p) => {
                    let (x, y) = transform.map(glyph.x + p.x, glyph.baseline - p.y);
                    builder.line_to(x, y);
                }
                Command::QuadTo(p1, p2) => {
                    let (x1, y1) = transform.map(glyph.x + p1.x, glyph.baseline - p1.y);
                    let (x2, y2) = transform.map(glyph.x + p2.x, glyph.baseline - p2.y);
                    builder.quad_to(x1, y1, x2, y2);
                }
                Command::CurveTo(p1, p2, p3) => {
                    let (x1, y1) = transform.map(glyph.x + p1.x, glyph.baseline - p1.y);
                    let (x2, y2) = transform.map(glyph.x + p2.x, glyph.baseline - p2.y);
                    let (x3, y3) = transform.map(glyph.x + p3.x, glyph.baseline - p3.y);
                    builder.cubic_to(x1, y1, x2, y2, x3, y3);
                }
                Command::Close => builder.close(),
            }
        }
    }
}

fn push_decoration_rect(
    builder: &mut PathBuilder,
    transform: &TextTransform,
    x_start: f32,
    y_start: f32,
    width: f32,
    height: f32,
) {
    let corners = [
        transform.map(x_start, y_start),
        transform.map(x_start + width, y_start),
        transform.map(x_start + width, y_start + height),
        transform.map(x_start, y_start + height),
    ];
    builder.move_to(corners[0].0, corners[0].1);
    for corner in &corners[1..] {
        builder.line_to(corner.0, corner.1);
    }
    builder.close();
}

fn push_decorations(
    builder: &mut PathBuilder,
    params: &TextMaskParams,
    canvas_height: f64,
    layout: &TextLayout,
    transform: &TextTransform,
) {
    if params.text_decoration == TextDecoration::None {
        return;
    }
    let scaled_font_size = params.font_size * (canvas_height / FONT_SIZE_SCALE_REFERENCE);
    let thickness = (scaled_font_size * TEXT_DECORATION_THICKNESS_RATIO).max(1.0) as f32;
    for line in &layout.lines {
        if line.width <= 0.0 {
            continue;
        }
        let x_start = -line.width / 2.0;
        match params.text_decoration {
            TextDecoration::Underline => {
                let y = line.y_center + line.descent + thickness;
                push_decoration_rect(builder, transform, x_start, y, line.width, thickness);
            }
            TextDecoration::LineThrough => {
                let y = line.y_center
                    - (line.ascent - line.descent) * STRIKETHROUGH_VERTICAL_RATIO as f32;
                push_decoration_rect(builder, transform, x_start, y, line.width, thickness);
            }
            TextDecoration::None => {}
        }
    }
}

pub fn render_body(params: &TextMaskParams, width: f64, height: f64, pixmap: &mut Pixmap) {
    TEXT_ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        let (font_system, cache) = &mut *engine;
        let layout = layout_text(font_system, params, height);
        let transform = TextTransform::new(params, width, height);
        let mut builder = PathBuilder::new();
        push_glyph_outlines(font_system, cache, &layout, &transform, &mut builder);
        push_decorations(&mut builder, params, height, &layout, &transform);
        paint::fill_white(&builder.finish(), pixmap);
    });
}

pub fn render_stroke(
    params: &TextMaskParams,
    width: f64,
    height: f64,
    pixmap: &mut Pixmap,
) -> Result<(), MaskRenderError> {
    let color = paint::parse_stroke_color(&params.stroke_color)?;
    TEXT_ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        let (font_system, cache) = &mut *engine;
        let layout = layout_text(font_system, params, height);
        if layout.glyphs.is_empty() {
            return;
        }
        let transform = TextTransform::new(params, width, height);
        let mut builder = PathBuilder::new();
        push_glyph_outlines(font_system, cache, &layout, &transform, &mut builder);
        paint::stroke_path_with(
            &builder.finish(),
            &paint::round_stroke(params.stroke_width * params.scale),
            color,
            pixmap,
        );

        if params.stroke_align != scene::StrokeAlign::Center {
            if let Some(mut fill) = Pixmap::new(pixmap.width(), pixmap.height()) {
                render_body(params, width, height, &mut fill);
                paint::apply_stroke_align(pixmap, &fill, params.stroke_align);
            }
        }
    });
    Ok(())
}
