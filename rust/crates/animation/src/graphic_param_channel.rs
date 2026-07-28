use scene::{ElementAnimations, ParamValues};
use time::MediaTime;

use crate::params::ParamDefinition;
use crate::resolve::resolve_animation_path_value_at_time;

pub const GRAPHIC_PARAM_PATH_PREFIX: &str = "params.";

pub fn build_graphic_param_path(param_key: &str) -> String {
    format!("{GRAPHIC_PARAM_PATH_PREFIX}{param_key}")
}

pub fn is_graphic_param_path(property_path: &str) -> bool {
    property_path.starts_with(GRAPHIC_PARAM_PATH_PREFIX)
}

pub fn parse_graphic_param_path(property_path: &str) -> Option<String> {
    let param_key = property_path.strip_prefix(GRAPHIC_PARAM_PATH_PREFIX)?;
    if param_key.is_empty() {
        return None;
    }
    Some(param_key.to_string())
}

pub fn resolve_graphic_params_at_time(
    params: &ParamValues,
    definitions: &[ParamDefinition],
    animations: Option<&ElementAnimations>,
    local_time: MediaTime,
) -> ParamValues {
    let mut resolved = params.clone();
    let safe_local_time = local_time.max(MediaTime::ZERO);

    for param in definitions {
        let path = build_graphic_param_path(param.key());
        if animations.and_then(|a| a.get(&path)).is_none() {
            continue;
        }
        let fallback_value = params
            .get(param.key())
            .cloned()
            .unwrap_or_else(|| param.default_value());
        resolved.insert(
            param.key().to_string(),
            resolve_animation_path_value_at_time(animations, &path, safe_local_time, &fallback_value),
        );
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{AnimationChannel, ChannelData, ParamValue, ScalarAnimationKey, ScalarChannel, ScalarSegmentType, TangentMode};
    use std::collections::BTreeMap;

    #[test]
    fn builds_and_parses_graphic_param_paths() {
        assert_eq!(build_graphic_param_path("blur"), "params.blur");
        assert!(is_graphic_param_path("params.blur"));
        assert!(!is_graphic_param_path("effects.x.params.y"));
        assert_eq!(parse_graphic_param_path("params.blur"), Some("blur".to_string()));
        assert_eq!(parse_graphic_param_path("params."), None);
        assert_eq!(parse_graphic_param_path("opacity"), None);
    }

    #[test]
    fn resolves_animated_params_over_static_values() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![ScalarAnimationKey {
                id: "a".to_string(),
                time: MediaTime::from_ticks(0),
                value: 5.0,
                left_handle: None,
                right_handle: None,
                segment_to_next: ScalarSegmentType::Linear,
                tangent_mode: TangentMode::Flat,
            }],
            extrapolation: None,
        });
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert("params.blur".to_string(), ChannelData::Single(channel));

        let mut params: ParamValues = BTreeMap::new();
        params.insert("blur".to_string(), ParamValue::Number(1.0));
        params.insert("other".to_string(), ParamValue::Number(2.0));

        let definitions = vec![
            ParamDefinition::number("blur", "Blur", 0.0, 0.0, None, 1.0),
            ParamDefinition::number("other", "Other", 0.0, 0.0, None, 1.0),
        ];
        let resolved = resolve_graphic_params_at_time(
            &params,
            &definitions,
            Some(&animations),
            MediaTime::from_ticks(0),
        );
        assert_eq!(resolved.get("blur"), Some(&ParamValue::Number(5.0)));
        assert_eq!(resolved.get("other"), Some(&ParamValue::Number(2.0)));
    }
}
