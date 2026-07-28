use scene::ParamValue;

use crate::channel_layout::{
    ChannelLayout, boolean_channel_layout, color_channel_layout, number_channel_layout,
    string_channel_layout,
};
use crate::types::{AnimationInterpolation, NumericSpec};

#[derive(Clone, Debug, PartialEq)]
pub struct NumberParamDefinition {
    pub key: String,
    pub label: String,
    pub default: f64,
    pub min: f64,
    pub max: Option<f64>,
    pub step: f64,
    pub channels: Option<ChannelLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BooleanParamDefinition {
    pub key: String,
    pub label: String,
    pub default: bool,
    pub channels: Option<ChannelLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorParamDefinition {
    pub key: String,
    pub label: String,
    pub default: String,
    pub channels: Option<ChannelLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectParamDefinition {
    pub key: String,
    pub label: String,
    pub default: String,
    pub options: Vec<SelectOption>,
    pub channels: Option<ChannelLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextParamDefinition {
    pub key: String,
    pub label: String,
    pub default: String,
    pub channels: Option<ChannelLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontParamDefinition {
    pub key: String,
    pub label: String,
    pub default: String,
    pub channels: Option<ChannelLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParamDefinition {
    Number(NumberParamDefinition),
    Boolean(BooleanParamDefinition),
    Color(ColorParamDefinition),
    Select(SelectParamDefinition),
    Text(TextParamDefinition),
    Font(FontParamDefinition),
}

impl ParamDefinition {
    pub fn number(
        key: &str,
        label: &str,
        default: f64,
        min: f64,
        max: Option<f64>,
        step: f64,
    ) -> Self {
        ParamDefinition::Number(NumberParamDefinition {
            key: key.to_string(),
            label: label.to_string(),
            default,
            min,
            max,
            step,
            channels: None,
        })
    }

    pub fn boolean(key: &str, label: &str, default: bool) -> Self {
        ParamDefinition::Boolean(BooleanParamDefinition {
            key: key.to_string(),
            label: label.to_string(),
            default,
            channels: None,
        })
    }

    pub fn color(key: &str, label: &str, default: &str) -> Self {
        ParamDefinition::Color(ColorParamDefinition {
            key: key.to_string(),
            label: label.to_string(),
            default: default.to_string(),
            channels: None,
        })
    }

    pub fn select(key: &str, label: &str, default: &str, options: Vec<SelectOption>) -> Self {
        ParamDefinition::Select(SelectParamDefinition {
            key: key.to_string(),
            label: label.to_string(),
            default: default.to_string(),
            options,
            channels: None,
        })
    }

    pub fn key(&self) -> &str {
        match self {
            ParamDefinition::Number(param) => &param.key,
            ParamDefinition::Boolean(param) => &param.key,
            ParamDefinition::Color(param) => &param.key,
            ParamDefinition::Select(param) => &param.key,
            ParamDefinition::Text(param) => &param.key,
            ParamDefinition::Font(param) => &param.key,
        }
    }

    pub fn default_value(&self) -> ParamValue {
        match self {
            ParamDefinition::Number(param) => ParamValue::Number(param.default),
            ParamDefinition::Boolean(param) => ParamValue::Bool(param.default),
            ParamDefinition::Color(param) => ParamValue::String(param.default.clone()),
            ParamDefinition::Select(param) => ParamValue::String(param.default.clone()),
            ParamDefinition::Text(param) => ParamValue::String(param.default.clone()),
            ParamDefinition::Font(param) => ParamValue::String(param.default.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParamValueKind {
    Number,
    Color,
    Discrete,
}

pub fn get_param_channel_layout(param: &ParamDefinition) -> ChannelLayout {
    match param {
        ParamDefinition::Number(param) => param
            .channels
            .clone()
            .unwrap_or_else(number_channel_layout),
        ParamDefinition::Boolean(param) => param
            .channels
            .clone()
            .unwrap_or_else(boolean_channel_layout),
        ParamDefinition::Color(param) => param.channels.clone().unwrap_or_else(color_channel_layout),
        ParamDefinition::Select(param) => param
            .channels
            .clone()
            .unwrap_or_else(string_channel_layout),
        ParamDefinition::Text(param) => param
            .channels
            .clone()
            .unwrap_or_else(string_channel_layout),
        ParamDefinition::Font(param) => param
            .channels
            .clone()
            .unwrap_or_else(string_channel_layout),
    }
}

pub fn get_param_value_kind(param: &ParamDefinition) -> ParamValueKind {
    let layout = get_param_channel_layout(param);
    match &layout {
        ChannelLayout::Composite(_) => ParamValueKind::Color,
        ChannelLayout::Leaf(component) => match component.value_kind {
            crate::channel_layout::ChannelValueKind::Scalar => ParamValueKind::Number,
            crate::channel_layout::ChannelValueKind::Discrete => ParamValueKind::Discrete,
        },
    }
}

pub fn get_param_default_interpolation(param: &ParamDefinition) -> AnimationInterpolation {
    let layout = get_param_channel_layout(param);
    layout
        .components()
        .first()
        .map(|component| component.default_interpolation)
        .unwrap_or(AnimationInterpolation::Linear)
}

pub fn get_param_numeric_range(param: &ParamDefinition) -> Option<NumericSpec> {
    let ParamDefinition::Number(param) = param else {
        return None;
    };
    Some(NumericSpec {
        min: Some(param.min),
        max: param.max,
        step: Some(param.step),
    })
}

fn fraction_digits_for_step(step: f64) -> usize {
    if step != 0.0 && step.abs() < 1e-6 {
        let exponential = format!("{step:e}");
        if let Some((_, exponent)) = exponential.split_once('e') {
            if let Ok(value) = exponent.parse::<i32>() {
                return value.unsigned_abs() as usize;
            }
        }
        return 0;
    }
    let normalized = format!("{step}");
    normalized
        .split_once('.')
        .map(|(_, fraction)| fraction.len())
        .unwrap_or(0)
}

pub fn snap_to_step(value: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return value;
    }
    let snapped = (value / step).round() * step;
    let digits = fraction_digits_for_step(step);
    format!("{snapped:.digits$}").parse().unwrap_or(snapped)
}

pub fn coerce_param_value(param: &ParamDefinition, value: &ParamValue) -> Option<ParamValue> {
    match param {
        ParamDefinition::Number(param) => {
            let ParamValue::Number(number) = value else {
                return None;
            };
            if number.is_nan() {
                return None;
            }
            let stepped = snap_to_step(*number, param.step);
            let max = param.max.unwrap_or(f64::INFINITY);
            Some(ParamValue::Number(stepped.min(max).max(param.min)))
        }
        ParamDefinition::Boolean(_) => match value {
            ParamValue::Bool(value) => Some(ParamValue::Bool(*value)),
            _ => None,
        },
        ParamDefinition::Color(_) | ParamDefinition::Text(_) | ParamDefinition::Font(_) => {
            match value {
                ParamValue::String(value) => Some(ParamValue::String(value.clone())),
                _ => None,
            }
        }
        ParamDefinition::Select(param) => match value {
            ParamValue::String(value)
                if param.options.iter().any(|option| &option.value == value) =>
            {
                Some(ParamValue::String(value.clone()))
            }
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intensity_param() -> ParamDefinition {
        ParamDefinition::number("intensity", "Intensity", 0.0, 0.0, Some(1.0), 0.25)
    }

    fn blend_param() -> ParamDefinition {
        ParamDefinition::select(
            "blend",
            "Blend",
            "normal",
            vec![
                SelectOption {
                    value: "normal".to_string(),
                    label: "Normal".to_string(),
                },
                SelectOption {
                    value: "multiply".to_string(),
                    label: "Multiply".to_string(),
                },
            ],
        )
    }

    #[test]
    fn snaps_and_clamps_number_params() {
        assert_eq!(
            coerce_param_value(&intensity_param(), &ParamValue::Number(0.62)),
            Some(ParamValue::Number(0.5))
        );
        assert_eq!(
            coerce_param_value(&intensity_param(), &ParamValue::Number(1.2)),
            Some(ParamValue::Number(1.0))
        );
    }

    #[test]
    fn rejects_nan_and_non_number_values() {
        assert_eq!(
            coerce_param_value(&intensity_param(), &ParamValue::Number(f64::NAN)),
            None
        );
        assert_eq!(
            coerce_param_value(&intensity_param(), &ParamValue::String("0.5".to_string())),
            None
        );
        assert_eq!(
            coerce_param_value(&intensity_param(), &ParamValue::Bool(true)),
            None
        );
    }

    #[test]
    fn passthrough_with_step_zero_guard() {
        let param = ParamDefinition::number("x", "X", 0.0, 0.0, None, 0.0);
        assert_eq!(
            coerce_param_value(&param, &ParamValue::Number(0.123)),
            Some(ParamValue::Number(0.123))
        );
    }

    #[test]
    fn accepts_valid_select_values() {
        assert_eq!(
            coerce_param_value(&blend_param(), &ParamValue::String("normal".to_string())),
            Some(ParamValue::String("normal".to_string()))
        );
        assert_eq!(
            coerce_param_value(&blend_param(), &ParamValue::String("multiply".to_string())),
            Some(ParamValue::String("multiply".to_string()))
        );
    }

    #[test]
    fn rejects_select_values_outside_options() {
        assert_eq!(
            coerce_param_value(&blend_param(), &ParamValue::String("screen".to_string())),
            None
        );
    }

    #[test]
    fn rejects_non_string_select_values() {
        assert_eq!(
            coerce_param_value(&blend_param(), &ParamValue::Number(42.0)),
            None
        );
        assert_eq!(coerce_param_value(&blend_param(), &ParamValue::Bool(true)), None);
    }

    #[test]
    fn boolean_params_accept_booleans_and_reject_other_types() {
        let param = ParamDefinition::boolean("visible", "Visible", true);
        assert_eq!(
            coerce_param_value(&param, &ParamValue::Bool(true)),
            Some(ParamValue::Bool(true))
        );
        assert_eq!(
            coerce_param_value(&param, &ParamValue::Bool(false)),
            Some(ParamValue::Bool(false))
        );
        assert_eq!(coerce_param_value(&param, &ParamValue::Number(1.0)), None);
        assert_eq!(
            coerce_param_value(&param, &ParamValue::String("true".to_string())),
            None
        );
    }

    #[test]
    fn color_params_accept_strings_and_reject_other_types() {
        let param = ParamDefinition::color("fill", "Fill", "#ffffff");
        assert_eq!(
            coerce_param_value(&param, &ParamValue::String("#ff0000".to_string())),
            Some(ParamValue::String("#ff0000".to_string()))
        );
        assert_eq!(
            coerce_param_value(&param, &ParamValue::Number(0xff0000 as f64)),
            None
        );
        assert_eq!(coerce_param_value(&param, &ParamValue::Bool(true)), None);
    }

    #[test]
    fn value_kind_maps_param_type_to_binding_kind() {
        let number = ParamDefinition::number("n", "N", 0.0, 0.0, None, 1.0);
        assert_eq!(get_param_value_kind(&number), ParamValueKind::Number);
        let color = ParamDefinition::color("c", "C", "#fff");
        assert_eq!(get_param_value_kind(&color), ParamValueKind::Color);
        let boolean = ParamDefinition::boolean("b", "B", false);
        assert_eq!(get_param_value_kind(&boolean), ParamValueKind::Discrete);
        assert_eq!(get_param_value_kind(&blend_param()), ParamValueKind::Discrete);
    }

    #[test]
    fn default_interpolation_is_linear_for_continuous_hold_for_discrete() {
        let number = ParamDefinition::number("n", "N", 0.0, 0.0, None, 1.0);
        assert_eq!(
            get_param_default_interpolation(&number),
            AnimationInterpolation::Linear
        );
        let color = ParamDefinition::color("c", "C", "#fff");
        assert_eq!(
            get_param_default_interpolation(&color),
            AnimationInterpolation::Linear
        );
        let boolean = ParamDefinition::boolean("b", "B", false);
        assert_eq!(
            get_param_default_interpolation(&boolean),
            AnimationInterpolation::Hold
        );
        assert_eq!(
            get_param_default_interpolation(&blend_param()),
            AnimationInterpolation::Hold
        );
    }

    #[test]
    fn numeric_range_returns_spec_for_number_params_only() {
        assert_eq!(
            get_param_numeric_range(&intensity_param()),
            Some(NumericSpec {
                min: Some(0.0),
                max: Some(1.0),
                step: Some(0.25),
            })
        );
        let color = ParamDefinition::color("c", "C", "#fff");
        assert_eq!(get_param_numeric_range(&color), None);
        let boolean = ParamDefinition::boolean("b", "B", false);
        assert_eq!(get_param_numeric_range(&boolean), None);
    }
}
