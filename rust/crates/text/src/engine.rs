use cosmic_text::fontdb::Source;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap};
use std::sync::Arc;

use crate::layout::{
    LineMetrics, ResolvedTextLayout, TextBlockMeasurement, measure_text_block, resolve_text_layout,
};
use crate::params::{FontStyle, FontWeight, TextLayoutParams};

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("failed to load font: {0}")]
    FontLoad(String),
}

pub struct TextEngine {
    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,
}

pub struct MeasuredTextLayout {
    pub resolved: ResolvedTextLayout,
    pub lines: Vec<String>,
    pub line_metrics: Vec<LineMetrics>,
    pub block: TextBlockMeasurement,
    pub(crate) buffer: Buffer,
}

impl TextEngine {
    pub fn new() -> Self {
        Self::from_font_system(FontSystem::new())
    }

    pub fn with_font_bytes(fonts: &[&[u8]]) -> Self {
        let sources = fonts
            .iter()
            .map(|bytes| Source::Binary(Arc::new(bytes.to_vec())));
        Self::from_font_system(FontSystem::new_with_fonts(sources))
    }

    fn from_font_system(font_system: FontSystem) -> Self {
        Self {
            font_system,
            swash_cache: SwashCache::new(),
        }
    }

    pub fn load_font_bytes(&mut self, bytes: &[u8]) -> Result<(), TextError> {
        let loaded = self
            .font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(bytes.to_vec())));
        if loaded.is_empty() {
            return Err(TextError::FontLoad(
                "no faces found in font data".to_string(),
            ));
        }
        Ok(())
    }

    pub fn measure(&mut self, params: &TextLayoutParams, canvas_height: f64) -> MeasuredTextLayout {
        self.measure_with_wrap(params, canvas_height, None)
    }

    pub fn measure_with_wrap(
        &mut self,
        params: &TextLayoutParams,
        canvas_height: f64,
        wrap_width: Option<f64>,
    ) -> MeasuredTextLayout {
        let resolved = resolve_text_layout(params, canvas_height);
        let mut buffer = self.build_buffer(params, &resolved, wrap_width);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut lines = Vec::new();
        let mut line_metrics = Vec::new();
        for line_i in 0..buffer.lines.len() {
            let line_text = buffer.lines[line_i].text().to_string();
            let layouts = buffer
                .line_layout(&mut self.font_system, line_i)
                .unwrap_or(&[]);
            let mut laid_out = false;
            for layout_line in layouts {
                laid_out = true;
                let (text, metrics) = visual_line_metrics(&line_text, layout_line, &resolved);
                lines.push(text);
                line_metrics.push(metrics);
            }
            if !laid_out {
                lines.push(line_text);
                line_metrics.push(empty_line_metrics(&resolved));
            }
        }
        if line_metrics.is_empty() {
            lines.push(String::new());
            line_metrics.push(empty_line_metrics(&resolved));
        }

        let block = measure_text_block(&line_metrics, resolved.line_height_px);
        MeasuredTextLayout {
            resolved,
            lines,
            line_metrics,
            block,
            buffer,
        }
    }

    fn build_buffer(
        &mut self,
        params: &TextLayoutParams,
        resolved: &ResolvedTextLayout,
        wrap_width: Option<f64>,
    ) -> Buffer {
        let metrics = Metrics::new(
            resolved.scaled_font_size as f32,
            resolved.line_height_px as f32,
        );
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let (wrap, width) = match wrap_width {
            Some(width) => (Wrap::WordOrGlyph, Some(width as f32)),
            None => (Wrap::None, None),
        };
        buffer.set_wrap(&mut self.font_system, wrap);
        buffer.set_size(&mut self.font_system, width, None);
        let attrs = Attrs::new()
            .family(Family::Name(&params.font_family))
            .weight(match params.font_weight {
                FontWeight::Normal => Weight::NORMAL,
                FontWeight::Bold => Weight::BOLD,
            })
            .style(match params.font_style {
                FontStyle::Normal => Style::Normal,
                FontStyle::Italic => Style::Italic,
            })
            .letter_spacing((resolved.letter_spacing / resolved.scaled_font_size.max(f64::EPSILON)) as f32);
        buffer.set_text(
            &mut self.font_system,
            &params.content,
            &attrs,
            Shaping::Advanced,
            None,
        );
        buffer
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn visual_line_metrics(
    line_text: &str,
    layout_line: &cosmic_text::LayoutLine,
    resolved: &ResolvedTextLayout,
) -> (String, LineMetrics) {
    if layout_line.glyphs.is_empty() {
        return (line_text.to_string(), empty_line_metrics(resolved));
    }
    let raw_width = layout_line
        .glyphs
        .iter()
        .fold(0.0_f32, |max, glyph| max.max(glyph.x + glyph.w));
    let width = (f64::from(raw_width) - resolved.letter_spacing).max(0.0);
    let start = layout_line
        .glyphs
        .iter()
        .map(|glyph| glyph.start)
        .min()
        .unwrap_or(0);
    let end = layout_line
        .glyphs
        .iter()
        .map(|glyph| glyph.end)
        .max()
        .unwrap_or(line_text.len());
    let text = line_text.get(start..end).unwrap_or(line_text).to_string();
    let metrics = LineMetrics {
        width,
        ascent: f64::from(layout_line.max_ascent),
        descent: f64::from(layout_line.max_descent),
    };
    (text, metrics)
}

fn empty_line_metrics(resolved: &ResolvedTextLayout) -> LineMetrics {
    LineMetrics {
        width: 0.0,
        ascent: resolved.scaled_font_size * 0.8,
        descent: resolved.scaled_font_size * 0.2,
    }
}
