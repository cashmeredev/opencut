use scene::ParamValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::color::{LinearRgba, parse_color_to_linear_rgba};
use crate::types::AnimationInterpolation;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChannelValueKind {
    Scalar,
    Discrete,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelComponentDefinition {
    pub key: String,
    pub value_kind: ChannelValueKind,
    pub default_interpolation: AnimationInterpolation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChannelLayout {
    Leaf(ChannelComponentDefinition),
    Composite(Vec<ChannelComponentDefinition>),
}

impl ChannelLayout {
    pub fn components(&self) -> &[ChannelComponentDefinition] {
        match self {
            ChannelLayout::Leaf(component) => std::slice::from_ref(component),
            ChannelLayout::Composite(components) => components,
        }
    }

    pub fn decompose(&self, value: &ParamValue) -> Option<BTreeMap<String, ParamValue>> {
        match self {
            ChannelLayout::Leaf(component) => {
                let mut values = BTreeMap::new();
                values.insert(component.key.clone(), value.clone());
                Some(values)
            }
            ChannelLayout::Composite(_) => {
                let ParamValue::String(color) = value else {
                    return None;
                };
                let LinearRgba { r, g, b, a } = parse_color_to_linear_rgba(color)?;
                let mut values = BTreeMap::new();
                values.insert("r".to_string(), ParamValue::Number(r));
                values.insert("g".to_string(), ParamValue::Number(g));
                values.insert("b".to_string(), ParamValue::Number(b));
                values.insert("a".to_string(), ParamValue::Number(a));
                Some(values)
            }
        }
    }
}

pub fn number_channel_layout() -> ChannelLayout {
    ChannelLayout::Leaf(ChannelComponentDefinition {
        key: "value".to_string(),
        value_kind: ChannelValueKind::Scalar,
        default_interpolation: AnimationInterpolation::Linear,
    })
}

pub fn boolean_channel_layout() -> ChannelLayout {
    ChannelLayout::Leaf(ChannelComponentDefinition {
        key: "value".to_string(),
        value_kind: ChannelValueKind::Discrete,
        default_interpolation: AnimationInterpolation::Hold,
    })
}

pub fn string_channel_layout() -> ChannelLayout {
    ChannelLayout::Leaf(ChannelComponentDefinition {
        key: "value".to_string(),
        value_kind: ChannelValueKind::Discrete,
        default_interpolation: AnimationInterpolation::Hold,
    })
}

pub fn color_channel_layout() -> ChannelLayout {
    let component = |key: &str| ChannelComponentDefinition {
        key: key.to_string(),
        value_kind: ChannelValueKind::Scalar,
        default_interpolation: AnimationInterpolation::Linear,
    };
    ChannelLayout::Composite(vec![
        component("r"),
        component("g"),
        component("b"),
        component("a"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_decompose_wraps_value_under_component_key() {
        let layout = number_channel_layout();
        let values = layout.decompose(&ParamValue::Number(3.0)).unwrap();
        assert_eq!(values.get("value"), Some(&ParamValue::Number(3.0)));
    }

    #[test]
    fn color_decompose_parses_hex_into_linear_components() {
        let layout = color_channel_layout();
        let values = layout
            .decompose(&ParamValue::String("#ff0000".to_string()))
            .unwrap();
        assert_eq!(values.get("r"), Some(&ParamValue::Number(1.0)));
        assert_eq!(values.get("g"), Some(&ParamValue::Number(0.0)));
        assert_eq!(values.get("a"), Some(&ParamValue::Number(1.0)));
        assert!(
            layout
                .decompose(&ParamValue::Number(1.0))
                .is_none()
        );
    }
}
