use scene::params::{ParamValue, ParamValues};

#[derive(Clone, Debug, PartialEq)]
pub struct ParamDependency {
    pub param: String,
    pub equals: ParamValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NumberParamDefinition {
    pub key: String,
    pub label: String,
    pub default_value: f64,
    pub min: f64,
    pub max: Option<f64>,
    pub step: f64,
    pub group: Option<String>,
    pub keyframable: Option<bool>,
    pub dependencies: Vec<ParamDependency>,
    pub display_multiplier: Option<f64>,
    pub unit: Option<String>,
    pub short_label: Option<String>,
}

impl NumberParamDefinition {
    pub fn new(
        key: &str,
        label: &str,
        default_value: f64,
        min: f64,
        max: Option<f64>,
        step: f64,
    ) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            default_value,
            min,
            max,
            step,
            group: None,
            keyframable: None,
            dependencies: Vec::new(),
            display_multiplier: None,
            unit: None,
            short_label: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BooleanParamDefinition {
    pub key: String,
    pub label: String,
    pub default_value: bool,
    pub group: Option<String>,
    pub keyframable: Option<bool>,
    pub dependencies: Vec<ParamDependency>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorParamDefinition {
    pub key: String,
    pub label: String,
    pub default_value: String,
    pub group: Option<String>,
    pub keyframable: Option<bool>,
    pub dependencies: Vec<ParamDependency>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectParamDefinition {
    pub key: String,
    pub label: String,
    pub default_value: String,
    pub options: Vec<SelectOption>,
    pub group: Option<String>,
    pub keyframable: Option<bool>,
    pub dependencies: Vec<ParamDependency>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextParamDefinition {
    pub key: String,
    pub label: String,
    pub default_value: String,
    pub group: Option<String>,
    pub keyframable: Option<bool>,
    pub dependencies: Vec<ParamDependency>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontParamDefinition {
    pub key: String,
    pub label: String,
    pub default_value: String,
    pub group: Option<String>,
    pub keyframable: Option<bool>,
    pub dependencies: Vec<ParamDependency>,
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
    pub fn key(&self) -> &str {
        match self {
            Self::Number(param) => &param.key,
            Self::Boolean(param) => &param.key,
            Self::Color(param) => &param.key,
            Self::Select(param) => &param.key,
            Self::Text(param) => &param.key,
            Self::Font(param) => &param.key,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Number(param) => &param.label,
            Self::Boolean(param) => &param.label,
            Self::Color(param) => &param.label,
            Self::Select(param) => &param.label,
            Self::Text(param) => &param.label,
            Self::Font(param) => &param.label,
        }
    }

    pub fn default_value(&self) -> ParamValue {
        match self {
            Self::Number(param) => ParamValue::Number(param.default_value),
            Self::Boolean(param) => ParamValue::Bool(param.default_value),
            Self::Color(param) => ParamValue::String(param.default_value.clone()),
            Self::Select(param) => ParamValue::String(param.default_value.clone()),
            Self::Text(param) => ParamValue::String(param.default_value.clone()),
            Self::Font(param) => ParamValue::String(param.default_value.clone()),
        }
    }

    pub fn numeric_range(&self) -> Option<(f64, Option<f64>, f64)> {
        match self {
            Self::Number(param) => Some((param.min, param.max, param.step)),
            _ => None,
        }
    }
}

pub fn build_default_param_values(params: &[ParamDefinition]) -> ParamValues {
    params
        .iter()
        .map(|param| (param.key().to_string(), param.default_value()))
        .collect()
}
