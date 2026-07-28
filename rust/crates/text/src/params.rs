use scene::params::{ParamValue, ParamValues};

pub const MIN_FONT_SIZE: f64 = 5.0;
pub const MAX_FONT_SIZE: f64 = 300.0;
pub const DEFAULT_TEXT_COLOR: &str = "#000000";
pub const FONT_SIZE_SCALE_REFERENCE: f64 = 90.0;
pub const CORNER_RADIUS_MIN: f64 = 0.0;
pub const CORNER_RADIUS_MAX: f64 = 100.0;
pub const DEFAULT_LETTER_SPACING: f64 = 0.0;
pub const DEFAULT_LINE_HEIGHT: f64 = 1.2;
pub const DEFAULT_BACKGROUND_ENABLED: bool = false;
pub const DEFAULT_BACKGROUND_COLOR: &str = "#000000";
pub const DEFAULT_BACKGROUND_CORNER_RADIUS: f64 = 0.0;
pub const DEFAULT_BACKGROUND_PADDING_X: f64 = 30.0;
pub const DEFAULT_BACKGROUND_PADDING_Y: f64 = 42.0;
pub const DEFAULT_BACKGROUND_OFFSET_X: f64 = 0.0;
pub const DEFAULT_BACKGROUND_OFFSET_Y: f64 = 0.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    LineThrough,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayoutParams {
    pub content: String,
    pub font_size: f64,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub letter_spacing: f64,
    pub line_height: f64,
}

impl Default for TextLayoutParams {
    fn default() -> Self {
        Self {
            content: "Default text".to_string(),
            font_size: 15.0,
            font_family: "Arial".to_string(),
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            text_align: TextAlign::Center,
            text_decoration: TextDecoration::None,
            letter_spacing: DEFAULT_LETTER_SPACING,
            line_height: DEFAULT_LINE_HEIGHT,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextBackground {
    pub enabled: bool,
    pub color: String,
    pub corner_radius: f64,
    pub padding_x: f64,
    pub padding_y: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl Default for TextBackground {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_BACKGROUND_ENABLED,
            color: DEFAULT_BACKGROUND_COLOR.to_string(),
            corner_radius: DEFAULT_BACKGROUND_CORNER_RADIUS,
            padding_x: DEFAULT_BACKGROUND_PADDING_X,
            padding_y: DEFAULT_BACKGROUND_PADDING_Y,
            offset_x: DEFAULT_BACKGROUND_OFFSET_X,
            offset_y: DEFAULT_BACKGROUND_OFFSET_Y,
        }
    }
}

pub fn text_layout_params_from(params: &ParamValues) -> TextLayoutParams {
    TextLayoutParams {
        content: string_param(params, "content", "Default text"),
        font_size: number_param(params, "fontSize", 15.0),
        font_family: string_param(params, "fontFamily", "Arial"),
        font_weight: match string_param(params, "fontWeight", "normal").as_str() {
            "bold" => FontWeight::Bold,
            _ => FontWeight::Normal,
        },
        font_style: match string_param(params, "fontStyle", "normal").as_str() {
            "italic" => FontStyle::Italic,
            _ => FontStyle::Normal,
        },
        text_align: match string_param(params, "textAlign", "center").as_str() {
            "left" => TextAlign::Left,
            "right" => TextAlign::Right,
            _ => TextAlign::Center,
        },
        text_decoration: match string_param(params, "textDecoration", "none").as_str() {
            "underline" => TextDecoration::Underline,
            "line-through" => TextDecoration::LineThrough,
            _ => TextDecoration::None,
        },
        letter_spacing: number_param(params, "letterSpacing", DEFAULT_LETTER_SPACING),
        line_height: number_param(params, "lineHeight", DEFAULT_LINE_HEIGHT),
    }
}

pub fn text_background_from(params: &ParamValues) -> TextBackground {
    TextBackground {
        enabled: bool_param(params, "background.enabled", DEFAULT_BACKGROUND_ENABLED),
        color: string_param(params, "background.color", DEFAULT_BACKGROUND_COLOR),
        corner_radius: number_param(
            params,
            "background.cornerRadius",
            DEFAULT_BACKGROUND_CORNER_RADIUS,
        ),
        padding_x: number_param(params, "background.paddingX", DEFAULT_BACKGROUND_PADDING_X),
        padding_y: number_param(params, "background.paddingY", DEFAULT_BACKGROUND_PADDING_Y),
        offset_x: number_param(params, "background.offsetX", DEFAULT_BACKGROUND_OFFSET_X),
        offset_y: number_param(params, "background.offsetY", DEFAULT_BACKGROUND_OFFSET_Y),
    }
}

fn string_param(params: &ParamValues, key: &str, fallback: &str) -> String {
    match params.get(key) {
        Some(ParamValue::String(value)) => value.clone(),
        _ => fallback.to_string(),
    }
}

fn number_param(params: &ParamValues, key: &str, fallback: f64) -> f64 {
    match params.get(key) {
        Some(ParamValue::Number(value)) => *value,
        _ => fallback,
    }
}

fn bool_param(params: &ParamValues, key: &str, fallback: bool) -> bool {
    match params.get(key) {
        Some(ParamValue::Bool(value)) => *value,
        _ => fallback,
    }
}
