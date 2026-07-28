use scene::{CurveHandle, ParamValue, ScalarSegmentType, TangentMode};
use serde::{Deserialize, Serialize};
use time::MediaTime;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AnimationInterpolation {
    Linear,
    Hold,
    Bezier,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NumericSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScalarCurveKeyframePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_handle: Option<Option<CurveHandle>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_handle: Option<Option<CurveHandle>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_to_next: Option<ScalarSegmentType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tangent_mode: Option<TangentMode>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ElementKeyframe {
    pub property_path: String,
    pub id: String,
    pub time: MediaTime,
    pub value: ParamValue,
    pub interpolation: AnimationInterpolation,
}
