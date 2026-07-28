use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::MediaTime;

use crate::params::ParamValue;

pub const ANIMATION_PROPERTY_PATHS: &[&str] = &[
    "transform.positionX",
    "transform.positionY",
    "transform.scaleX",
    "transform.scaleY",
    "transform.rotate",
    "opacity",
    "volume",
    "color",
    "background.color",
    "background.paddingX",
    "background.paddingY",
    "background.offsetX",
    "background.offsetY",
    "background.cornerRadius",
];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScalarSegmentType {
    Step,
    Linear,
    Bezier,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TangentMode {
    Auto,
    Aligned,
    Broken,
    Flat,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChannelExtrapolationMode {
    Hold,
    Linear,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct CurveHandle {
    pub dt: MediaTime,
    pub dv: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScalarAnimationKey {
    pub id: String,
    pub time: MediaTime,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_handle: Option<CurveHandle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_handle: Option<CurveHandle>,
    pub segment_to_next: ScalarSegmentType,
    pub tangent_mode: TangentMode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum DiscreteValue {
    Bool(bool),
    String(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiscreteAnimationKey {
    pub id: String,
    pub time: MediaTime,
    pub value: DiscreteValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelExtrapolation {
    pub before: ChannelExtrapolationMode,
    pub after: ChannelExtrapolationMode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ScalarChannel {
    pub keys: Vec<ScalarAnimationKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extrapolation: Option<ChannelExtrapolation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiscreteChannel {
    pub keys: Vec<DiscreteAnimationKey>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum AnimationChannel {
    Scalar(ScalarChannel),
    Discrete(DiscreteChannel),
}

pub type CompositeChannelData = BTreeMap<String, AnimationChannel>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ChannelData {
    Single(AnimationChannel),
    Composite(CompositeChannelData),
}

pub type ElementAnimations = BTreeMap<String, ChannelData>;

impl From<DiscreteValue> for ParamValue {
    fn from(value: DiscreteValue) -> Self {
        match value {
            DiscreteValue::Bool(b) => ParamValue::Bool(b),
            DiscreteValue::String(s) => ParamValue::String(s),
        }
    }
}
