mod color;
mod engine;
mod layout;
mod params;
mod raster;

pub use color::Color;
pub use engine::{MeasuredTextLayout, TextEngine, TextError};
pub use layout::{
    LineMetrics, Rect, ResolvedTextLayout, STRIKETHROUGH_VERTICAL_RATIO,
    TEXT_DECORATION_THICKNESS_RATIO, TextBlockMeasurement, align_offset, decoration_rect,
    measure_text_block, resolve_text_layout, text_background_rect, text_rect, text_visual_rect,
};
pub use params::{
    CORNER_RADIUS_MAX, CORNER_RADIUS_MIN, DEFAULT_BACKGROUND_COLOR,
    DEFAULT_BACKGROUND_CORNER_RADIUS, DEFAULT_BACKGROUND_ENABLED, DEFAULT_BACKGROUND_OFFSET_X,
    DEFAULT_BACKGROUND_OFFSET_Y, DEFAULT_BACKGROUND_PADDING_X, DEFAULT_BACKGROUND_PADDING_Y,
    DEFAULT_LETTER_SPACING, DEFAULT_LINE_HEIGHT, DEFAULT_TEXT_COLOR, FONT_SIZE_SCALE_REFERENCE,
    FontStyle, FontWeight, MAX_FONT_SIZE, MIN_FONT_SIZE, TextAlign, TextBackground, TextDecoration,
    TextLayoutParams, text_background_from, text_layout_params_from,
};
pub use raster::{RasterOptions, RgbaBuffer, StrokeStyle, rasterize};
