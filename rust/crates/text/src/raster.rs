use crate::color::Color;
use crate::engine::{MeasuredTextLayout, TextEngine};
use crate::layout::{align_offset, decoration_rect, text_background_rect, text_visual_rect};
use crate::params::TextBackground;

#[derive(Clone, Debug, PartialEq)]
pub struct RgbaBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbaBuffer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> Color {
        let index = (y as usize * self.width as usize + x as usize) * 4;
        Color::rgba(
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
            self.pixels[index + 3],
        )
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || color.a == 0 {
            return;
        }
        let index = (y as usize * self.width as usize + x as usize) * 4;
        let src_alpha = u32::from(color.a);
        let dst_alpha = u32::from(self.pixels[index + 3]);
        let out_alpha = src_alpha + dst_alpha * (255 - src_alpha) / 255;
        if out_alpha == 0 {
            return;
        }
        for (channel, src) in [(0, color.r), (1, color.g), (2, color.b)] {
            let dst = u32::from(self.pixels[index + channel]);
            let blended = (u32::from(src) * src_alpha * 255
                + dst * dst_alpha * (255 - src_alpha))
                / (out_alpha * 255);
            self.pixels[index + channel] = blended as u8;
        }
        self.pixels[index + 3] = out_alpha as u8;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct RasterOptions<'a> {
    pub text_color: Color,
    pub background: Option<(&'a TextBackground, Color)>,
    pub stroke: Option<StrokeStyle>,
}

pub fn rasterize(
    engine: &mut TextEngine,
    measured: &MeasuredTextLayout,
    options: &RasterOptions,
) -> RgbaBuffer {
    let resolved = &measured.resolved;
    let disabled_background = TextBackground::default();
    let rect_background = options
        .background
        .map(|(background, _)| background)
        .unwrap_or(&disabled_background);
    let visual = text_visual_rect(
        resolved.text_align,
        &measured.block,
        rect_background,
        resolved.font_size_ratio,
    );
    let width = visual.width.ceil().max(1.0) as u32;
    let height = visual.height.ceil().max(1.0) as u32;
    let mut output = RgbaBuffer::new(width, height);
    let origin_x = -visual.left;
    let origin_y = -visual.top;

    if let Some((background, color)) = options.background {
        if color.a > 0 {
            if let Some(rect) = text_background_rect(
                resolved.text_align,
                &measured.block,
                background,
                resolved.font_size_ratio,
            ) {
                let radius = (background.corner_radius.clamp(
                    crate::params::CORNER_RADIUS_MIN,
                    crate::params::CORNER_RADIUS_MAX,
                ) / 100.0)
                    * (rect.width.min(rect.height) / 2.0);
                fill_rounded_rect(
                    &mut output,
                    origin_x + rect.left,
                    origin_y + rect.top,
                    rect.width,
                    rect.height,
                    radius,
                    color,
                );
            }
        }
    }

    if options.text_color.a > 0 || options.stroke.is_some_and(|stroke| stroke.color.a > 0) {
        draw_glyphs(engine, measured, &mut output, origin_x, origin_y, options);
    }

    if options.text_color.a > 0 {
        for (index, metrics) in measured.line_metrics.iter().enumerate() {
            let line_y =
                index as f64 * resolved.line_height_px - measured.block.visual_center_offset;
            if let Some(rect) = decoration_rect(
                resolved.text_decoration,
                resolved.text_align,
                metrics.width,
                line_y,
                metrics.ascent,
                metrics.descent,
                resolved.scaled_font_size,
            ) {
                fill_rect(
                    &mut output,
                    origin_x + rect.left,
                    origin_y + rect.top,
                    rect.width,
                    rect.height,
                    options.text_color,
                );
            }
        }
    }

    output
}

fn draw_glyphs(
    engine: &mut TextEngine,
    measured: &MeasuredTextLayout,
    output: &mut RgbaBuffer,
    origin_x: f64,
    origin_y: f64,
    options: &RasterOptions,
) {
    let resolved = &measured.resolved;
    let y_shift = origin_y - measured.block.height / 2.0;
    for run in measured.buffer.layout_runs() {
        let raw_width = run
            .glyphs
            .iter()
            .fold(0.0_f32, |max, glyph| max.max(glyph.x + glyph.w));
        let line_width = (f64::from(raw_width) - resolved.letter_spacing).max(0.0);
        let x_offset = origin_x + align_offset(resolved.text_align, line_width);
        for glyph in run.glyphs {
            let physical = glyph.physical((x_offset as f32, y_shift as f32), 1.0);
            let baseline = run.line_y as i32;
            if let Some(stroke) = options.stroke {
                if stroke.color.a > 0 && stroke.width > 0.0 {
                    let cosmic = stroke.color.to_cosmic();
                    let offsets = stroke_offsets(stroke.width / 2.0);
                    engine.swash_cache.with_pixels(
                        &mut engine.font_system,
                        physical.cache_key,
                        cosmic,
                        |x, y, color| {
                            for (dx, dy) in &offsets {
                                output.blend_pixel(
                                    physical.x + x + dx,
                                    baseline + physical.y + y + dy,
                                    Color::from_cosmic(color),
                                );
                            }
                        },
                    );
                }
            }
            if options.text_color.a > 0 {
                let cosmic = options.text_color.to_cosmic();
                engine.swash_cache.with_pixels(
                    &mut engine.font_system,
                    physical.cache_key,
                    cosmic,
                    |x, y, color| {
                        output.blend_pixel(
                            physical.x + x,
                            baseline + physical.y + y,
                            Color::from_cosmic(color),
                        );
                    },
                );
            }
        }
    }
}

fn stroke_offsets(radius: f64) -> Vec<(i32, i32)> {
    let mut offsets = Vec::new();
    if radius < 0.5 {
        offsets.push((0, 0));
        return offsets;
    }
    let steps = ((radius * 6.0).ceil() as u32).max(16);
    for step in 0..steps {
        let angle = std::f64::consts::TAU * f64::from(step) / f64::from(steps);
        let dx = (radius * angle.cos()).round() as i32;
        let dy = (radius * angle.sin()).round() as i32;
        if !offsets.contains(&(dx, dy)) {
            offsets.push((dx, dy));
        }
    }
    offsets
}

fn fill_rect(output: &mut RgbaBuffer, left: f64, top: f64, width: f64, height: f64, color: Color) {
    let x_start = left.round() as i32;
    let y_start = top.round() as i32;
    let x_end = (left + width).round() as i32;
    let y_end = (top + height).round() as i32;
    for y in y_start..y_end {
        for x in x_start..x_end {
            output.blend_pixel(x, y, color);
        }
    }
}

fn fill_rounded_rect(
    output: &mut RgbaBuffer,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    radius: f64,
    color: Color) {
    let radius = radius.clamp(0.0, width.min(height) / 2.0);
    let center_x = left + width / 2.0;
    let center_y = top + height / 2.0;
    let inner_x = (width / 2.0 - radius).max(0.0);
    let inner_y = (height / 2.0 - radius).max(0.0);
    let x_start = left.floor() as i32;
    let y_start = top.floor() as i32;
    let x_end = (left + width).ceil() as i32;
    let y_end = (top + height).ceil() as i32;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let qx = (f64::from(x) + 0.5 - center_x).abs() - inner_x;
            let qy = (f64::from(y) + 0.5 - center_y).abs() - inner_y;
            let outside = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
            let coverage = (0.5 - outside).clamp(0.0, 1.0);
            if coverage > 0.0 {
                output.blend_pixel(x, y, color.scaled_alpha(coverage));
            }
        }
    }
}
