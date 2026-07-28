use scene::{ElementAnimations, ParamValues};
use time::MediaTime;

use crate::keyframes::remove_element_keyframe;
use crate::resolve::resolve_animation_path_value_at_time;

pub const EFFECT_PARAM_PATH_PREFIX: &str = "effects.";
pub const EFFECT_PARAM_PATH_SUFFIX: &str = ".params.";

pub fn build_effect_param_path(effect_id: &str, param_key: &str) -> String {
    format!("{EFFECT_PARAM_PATH_PREFIX}{effect_id}{EFFECT_PARAM_PATH_SUFFIX}{param_key}")
}

pub fn is_effect_param_path(property_path: &str) -> bool {
    property_path.starts_with(EFFECT_PARAM_PATH_PREFIX)
        && property_path.contains(EFFECT_PARAM_PATH_SUFFIX)
}

pub fn parse_effect_param_path(property_path: &str) -> Option<(String, String)> {
    if !is_effect_param_path(property_path) {
        return None;
    }
    let without_prefix = &property_path[EFFECT_PARAM_PATH_PREFIX.len()..];
    let separator_index = without_prefix.find(EFFECT_PARAM_PATH_SUFFIX)?;
    if separator_index == 0 {
        return None;
    }
    let effect_id = &without_prefix[..separator_index];
    let param_key = &without_prefix[separator_index + EFFECT_PARAM_PATH_SUFFIX.len()..];
    if effect_id.is_empty() || param_key.is_empty() {
        return None;
    }
    Some((effect_id.to_string(), param_key.to_string()))
}

pub fn resolve_effect_params_at_time(
    effect_id: &str,
    params: &ParamValues,
    animations: Option<&ElementAnimations>,
    local_time: MediaTime,
) -> ParamValues {
    let safe_local_time = local_time.max(MediaTime::ZERO);
    let mut resolved = ParamValues::new();

    for (param_key, static_value) in params {
        let path = build_effect_param_path(effect_id, param_key);
        let value = if animations.and_then(|a| a.get(&path)).is_some() {
            resolve_animation_path_value_at_time(animations, &path, safe_local_time, static_value)
        } else {
            static_value.clone()
        };
        resolved.insert(param_key.clone(), value);
    }

    resolved
}

pub fn remove_effect_param_keyframe(
    animations: Option<&ElementAnimations>,
    effect_id: &str,
    param_key: &str,
    keyframe_id: &str,
) -> Option<ElementAnimations> {
    remove_element_keyframe(
        animations,
        &build_effect_param_path(effect_id, param_key),
        keyframe_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{AnimationChannel, ChannelData, ParamValue, ScalarAnimationKey, ScalarChannel, ScalarSegmentType, TangentMode};
    use std::collections::BTreeMap;

    #[test]
    fn builds_and_parses_effect_param_paths() {
        let path = build_effect_param_path("glow", "intensity");
        assert_eq!(path, "effects.glow.params.intensity");
        assert!(is_effect_param_path(&path));
        assert!(!is_effect_param_path("params.blur"));
        assert_eq!(
            parse_effect_param_path(&path),
            Some(("glow".to_string(), "intensity".to_string()))
        );
        assert_eq!(parse_effect_param_path("effects..params.x"), None);
        assert_eq!(parse_effect_param_path("effects.glow.params."), None);
        assert_eq!(parse_effect_param_path("opacity"), None);
    }

    #[test]
    fn resolves_effect_params_and_removes_keyframes() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![ScalarAnimationKey {
                id: "k1".to_string(),
                time: MediaTime::from_ticks(0),
                value: 3.0,
                left_handle: None,
                right_handle: None,
                segment_to_next: ScalarSegmentType::Linear,
                tangent_mode: TangentMode::Flat,
            }],
            extrapolation: None,
        });
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert(
            "effects.glow.params.intensity".to_string(),
            ChannelData::Single(channel),
        );

        let mut params: ParamValues = BTreeMap::new();
        params.insert("intensity".to_string(), ParamValue::Number(1.0));
        let resolved = resolve_effect_params_at_time(
            "glow",
            &params,
            Some(&animations),
            MediaTime::from_ticks(0),
        );
        assert_eq!(resolved.get("intensity"), Some(&ParamValue::Number(3.0)));

        let after = remove_effect_param_keyframe(Some(&animations), "glow", "intensity", "k1");
        assert!(after.is_none());
    }
}
