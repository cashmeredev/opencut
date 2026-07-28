use scene::{ParamValue, ParamValues, TextElement};
use time::MediaTime;

use crate::creation::DEFAULT_NEW_ELEMENT_DURATION;

pub const DEFAULT_OPACITY: f64 = 1.0;
pub const DEFAULT_BLEND_MODE: &str = "normal";
pub const DEFAULT_VOLUME: f64 = 0.0;
pub const DEFAULT_TRANSFORM_SCALE: f64 = 1.0;
pub const DEFAULT_TRANSFORM_POSITION: f64 = 0.0;
pub const DEFAULT_TRANSFORM_ROTATE: f64 = 0.0;

pub const DEFAULT_TEXT_LETTER_SPACING: f64 = 0.0;
pub const DEFAULT_TEXT_LINE_HEIGHT: f64 = 1.2;

#[derive(Clone, Debug, PartialEq)]
pub struct DefaultTextBackground {
    pub enabled: bool,
    pub color: String,
    pub corner_radius: f64,
    pub padding_x: f64,
    pub padding_y: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

pub fn default_text_background() -> DefaultTextBackground {
    DefaultTextBackground {
        enabled: false,
        color: "#000000".to_string(),
        corner_radius: 0.0,
        padding_x: 30.0,
        padding_y: 42.0,
        offset_x: 0.0,
        offset_y: 0.0,
    }
}

pub fn default_text_params() -> ParamValues {
    let background = default_text_background();
    ParamValues::from([
        ("content".to_string(), ParamValue::String("Default text".to_string())),
        ("fontSize".to_string(), ParamValue::Number(15.0)),
        ("fontFamily".to_string(), ParamValue::String("Arial".to_string())),
        ("color".to_string(), ParamValue::String("#ffffff".to_string())),
        ("textAlign".to_string(), ParamValue::String("center".to_string())),
        ("fontWeight".to_string(), ParamValue::String("normal".to_string())),
        ("fontStyle".to_string(), ParamValue::String("normal".to_string())),
        ("textDecoration".to_string(), ParamValue::String("none".to_string())),
        (
            "letterSpacing".to_string(),
            ParamValue::Number(DEFAULT_TEXT_LETTER_SPACING),
        ),
        (
            "lineHeight".to_string(),
            ParamValue::Number(DEFAULT_TEXT_LINE_HEIGHT),
        ),
        (
            "background.enabled".to_string(),
            ParamValue::Bool(background.enabled),
        ),
        (
            "background.color".to_string(),
            ParamValue::String(background.color),
        ),
        (
            "background.cornerRadius".to_string(),
            ParamValue::Number(background.corner_radius),
        ),
        (
            "background.paddingX".to_string(),
            ParamValue::Number(background.padding_x),
        ),
        (
            "background.paddingY".to_string(),
            ParamValue::Number(background.padding_y),
        ),
        (
            "background.offsetX".to_string(),
            ParamValue::Number(background.offset_x),
        ),
        (
            "background.offsetY".to_string(),
            ParamValue::Number(background.offset_y),
        ),
        (
            "transform.positionX".to_string(),
            ParamValue::Number(DEFAULT_TRANSFORM_POSITION),
        ),
        (
            "transform.positionY".to_string(),
            ParamValue::Number(DEFAULT_TRANSFORM_POSITION),
        ),
        (
            "transform.scaleX".to_string(),
            ParamValue::Number(DEFAULT_TRANSFORM_SCALE),
        ),
        (
            "transform.scaleY".to_string(),
            ParamValue::Number(DEFAULT_TRANSFORM_SCALE),
        ),
        (
            "transform.rotate".to_string(),
            ParamValue::Number(DEFAULT_TRANSFORM_ROTATE),
        ),
        ("opacity".to_string(), ParamValue::Number(DEFAULT_OPACITY)),
        (
            "blendMode".to_string(),
            ParamValue::String(DEFAULT_BLEND_MODE.to_string()),
        ),
    ])
}

pub fn default_text_element(id: impl Into<String>) -> TextElement {
    TextElement {
        base: scene::BaseTimelineElement {
            id: id.into(),
            name: "Text".to_string(),
            duration: DEFAULT_NEW_ELEMENT_DURATION,
            start_time: MediaTime::ZERO,
            trim_start: MediaTime::ZERO,
            trim_end: MediaTime::ZERO,
            source_duration: None,
            animations: None,
            params: default_text_params(),
        },
        hidden: None,
        effects: None,
    }
}
