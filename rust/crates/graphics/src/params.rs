use scene::params::{ParamValue, ParamValues};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamGroup {
    Stroke,
}

impl ParamGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamGroup::Stroke => "stroke",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamDefinition {
    Number {
        key: &'static str,
        label: &'static str,
        group: Option<ParamGroup>,
        default: f64,
        min: f64,
        max: Option<f64>,
        step: f64,
        short_label: Option<&'static str>,
    },
    Color {
        key: &'static str,
        label: &'static str,
        group: Option<ParamGroup>,
        default: &'static str,
    },
    Select {
        key: &'static str,
        label: &'static str,
        group: Option<ParamGroup>,
        default: &'static str,
        options: &'static [SelectOption],
    },
}

impl ParamDefinition {
    pub fn key(&self) -> &'static str {
        match self {
            ParamDefinition::Number { key, .. } => key,
            ParamDefinition::Color { key, .. } => key,
            ParamDefinition::Select { key, .. } => key,
        }
    }

    pub fn default_value(&self) -> ParamValue {
        match self {
            ParamDefinition::Number { default, .. } => ParamValue::Number(*default),
            ParamDefinition::Color { default, .. } => ParamValue::String((*default).to_string()),
            ParamDefinition::Select { default, .. } => ParamValue::String((*default).to_string()),
        }
    }
}

pub fn build_default_param_values(params: &[ParamDefinition]) -> ParamValues {
    params
        .iter()
        .map(|param| (param.key().to_string(), param.default_value()))
        .collect()
}
