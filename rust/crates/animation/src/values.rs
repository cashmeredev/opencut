use scene::ElementAnimations;
use time::MediaTime;

use crate::resolve::resolve_animation_path_value_at_time;
use scene::ParamValue;

pub fn resolve_opacity_at_time(
    base_opacity: f64,
    animations: Option<&ElementAnimations>,
    local_time: MediaTime,
) -> f64 {
    match resolve_animation_path_value_at_time(
        animations,
        "opacity",
        local_time.max(MediaTime::ZERO),
        &ParamValue::Number(base_opacity),
    ) {
        ParamValue::Number(value) => value,
        _ => base_opacity,
    }
}

pub fn resolve_number_at_time(
    base_value: f64,
    animations: Option<&ElementAnimations>,
    property_path: &str,
    local_time: MediaTime,
) -> f64 {
    match resolve_animation_path_value_at_time(
        animations,
        property_path,
        local_time.max(MediaTime::ZERO),
        &ParamValue::Number(base_value),
    ) {
        ParamValue::Number(value) => value,
        _ => base_value,
    }
}

pub fn resolve_color_at_time(
    base_color: &str,
    animations: Option<&ElementAnimations>,
    property_path: &str,
    local_time: MediaTime,
) -> String {
    match resolve_animation_path_value_at_time(
        animations,
        property_path,
        local_time.max(MediaTime::ZERO),
        &ParamValue::String(base_color.to_string()),
    ) {
        ParamValue::String(value) => value,
        _ => base_color.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{AnimationChannel, ChannelData, ScalarAnimationKey, ScalarChannel, ScalarSegmentType, TangentMode};
    use std::collections::BTreeMap;

    #[test]
    fn resolves_opacity_with_animation_and_fallback() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![ScalarAnimationKey {
                id: "a".to_string(),
                time: MediaTime::from_ticks(0),
                value: 0.25,
                left_handle: None,
                right_handle: None,
                segment_to_next: ScalarSegmentType::Linear,
                tangent_mode: TangentMode::Flat,
            }],
            extrapolation: None,
        });
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert("opacity".to_string(), ChannelData::Single(channel));
        assert_eq!(
            resolve_opacity_at_time(1.0, Some(&animations), MediaTime::from_ticks(0)),
            0.25
        );
        assert_eq!(
            resolve_opacity_at_time(1.0, Some(&animations), MediaTime::from_ticks(-5)),
            0.25
        );
        assert_eq!(resolve_opacity_at_time(1.0, None, MediaTime::from_ticks(0)), 1.0);
    }
}
