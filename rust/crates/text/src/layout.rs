use crate::params::{
    FONT_SIZE_SCALE_REFERENCE, TextAlign, TextBackground, TextDecoration, TextLayoutParams,
};

pub const TEXT_DECORATION_THICKNESS_RATIO: f64 = 0.07;
pub const STRIKETHROUGH_VERTICAL_RATIO: f64 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedTextLayout {
    pub scaled_font_size: f64,
    pub letter_spacing: f64,
    pub line_height_px: f64,
    pub font_size_ratio: f64,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
}

pub fn resolve_text_layout(text: &TextLayoutParams, canvas_height: f64) -> ResolvedTextLayout {
    let scaled_font_size = text.font_size * (canvas_height / FONT_SIZE_SCALE_REFERENCE);
    ResolvedTextLayout {
        scaled_font_size,
        letter_spacing: text.letter_spacing,
        line_height_px: scaled_font_size * text.line_height,
        font_size_ratio: text.font_size / 15.0,
        text_align: text.text_align,
        text_decoration: text.text_decoration,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LineMetrics {
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextBlockMeasurement {
    pub visual_center_offset: f64,
    pub height: f64,
    pub max_width: f64,
}

pub fn measure_text_block(
    line_metrics: &[LineMetrics],
    line_height_px: f64,
) -> TextBlockMeasurement {
    let max_width = line_metrics
        .iter()
        .fold(0.0_f64, |max, metrics| max.max(metrics.width));
    let line_count = line_metrics.len() as f64;
    TextBlockMeasurement {
        visual_center_offset: ((line_count - 1.0) * line_height_px) / 2.0,
        height: line_count * line_height_px,
        max_width,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn right(self) -> f64 {
        self.left + self.width
    }

    pub fn bottom(self) -> f64 {
        self.top + self.height
    }
}

pub fn align_offset(text_align: TextAlign, line_width: f64) -> f64 {
    match text_align {
        TextAlign::Left => 0.0,
        TextAlign::Center => -line_width / 2.0,
        TextAlign::Right => -line_width,
    }
}

pub fn text_rect(text_align: TextAlign, block: &TextBlockMeasurement) -> Rect {
    Rect {
        left: align_offset(text_align, block.max_width),
        top: -block.height / 2.0,
        width: block.max_width,
        height: block.height,
    }
}

fn background_visible(background: &TextBackground) -> bool {
    background.enabled && !background.color.is_empty() && background.color != "transparent"
}

pub fn text_background_rect(
    text_align: TextAlign,
    block: &TextBlockMeasurement,
    background: &TextBackground,
    font_size_ratio: f64,
) -> Option<Rect> {
    if !background_visible(background) {
        return None;
    }
    let rect = text_rect(text_align, block);
    let padding_x = background.padding_x * font_size_ratio;
    let padding_y = background.padding_y * font_size_ratio;
    Some(Rect {
        left: rect.left - padding_x + background.offset_x,
        top: rect.top - padding_y + background.offset_y,
        width: rect.width + padding_x * 2.0,
        height: rect.height + padding_y * 2.0,
    })
}

pub fn text_visual_rect(
    text_align: TextAlign,
    block: &TextBlockMeasurement,
    background: &TextBackground,
    font_size_ratio: f64,
) -> Rect {
    let rect = text_rect(text_align, block);
    let Some(background_rect) = text_background_rect(text_align, block, background, font_size_ratio)
    else {
        return rect;
    };
    let left = rect.left.min(background_rect.left);
    let top = rect.top.min(background_rect.top);
    Rect {
        left,
        top,
        width: rect.right().max(background_rect.right()) - left,
        height: rect.bottom().max(background_rect.bottom()) - top,
    }
}

pub fn decoration_rect(
    text_decoration: TextDecoration,
    text_align: TextAlign,
    line_width: f64,
    line_y: f64,
    ascent: f64,
    descent: f64,
    scaled_font_size: f64,
) -> Option<Rect> {
    if text_decoration == TextDecoration::None {
        return None;
    }
    let thickness = (scaled_font_size * TEXT_DECORATION_THICKNESS_RATIO).max(1.0);
    let top = match text_decoration {
        TextDecoration::Underline => line_y + descent + thickness,
        TextDecoration::LineThrough => line_y - (ascent - descent) * STRIKETHROUGH_VERTICAL_RATIO,
        TextDecoration::None => unreachable!(),
    };
    Some(Rect {
        left: align_offset(text_align, line_width),
        top,
        width: line_width,
        height: thickness,
    })
}
