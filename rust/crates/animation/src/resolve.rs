use scene::{ChannelData, ElementAnimations, ParamValue};
use time::MediaTime;

use crate::color::{LinearRgba, format_linear_rgba, parse_color_to_linear_rgba};
use crate::interpolation::channel_value_at_time;

pub fn get_element_local_time(
    timeline_time: MediaTime,
    element_start_time: MediaTime,
    element_duration: MediaTime,
) -> MediaTime {
    let local_time = timeline_time - element_start_time;
    if local_time <= MediaTime::ZERO {
        return MediaTime::ZERO;
    }
    if local_time >= element_duration {
        return element_duration;
    }
    local_time
}

pub fn resolve_animation_path_value_at_time(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    local_time: MediaTime,
    fallback_value: &ParamValue,
) -> ParamValue {
    let Some(data) = animations.and_then(|a| a.get(property_path)) else {
        return fallback_value.clone();
    };

    match data {
        ChannelData::Single(channel) => match channel_value_at_time(
            Some(channel),
            local_time,
            fallback_value,
        ) {
            value => value,
        },
        ChannelData::Composite(components) => {
            let ParamValue::String(fallback_color) = fallback_value else {
                return fallback_value.clone();
            };
            let (Some(r), Some(g), Some(b), Some(a)) = (
                components.get("r"),
                components.get("g"),
                components.get("b"),
                components.get("a"),
            ) else {
                return fallback_value.clone();
            };
            let Some(fallback_components) = parse_color_to_linear_rgba(fallback_color) else {
                return fallback_value.clone();
            };

            let component_at = |channel: &scene::AnimationChannel, fallback: f64| {
                match channel_value_at_time(
                    Some(channel),
                    local_time,
                    &ParamValue::Number(fallback),
                ) {
                    ParamValue::Number(value) => value,
                    _ => fallback,
                }
            };
            ParamValue::String(format_linear_rgba(&LinearRgba {
                r: component_at(r, fallback_components.r),
                g: component_at(g, fallback_components.g),
                b: component_at(b, fallback_components.b),
                a: component_at(a, fallback_components.a),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{AnimationChannel, ScalarAnimationKey, ScalarChannel, ScalarSegmentType, TangentMode};
    use std::collections::BTreeMap;

    fn at(time: i64) -> MediaTime {
        MediaTime::from_ticks(time)
    }

    #[test]
    fn local_time_clamps_to_element_range() {
        assert_eq!(get_element_local_time(at(50), at(100), at(200)), at(0));
        assert_eq!(get_element_local_time(at(150), at(100), at(200)), at(50));
        assert_eq!(get_element_local_time(at(400), at(100), at(200)), at(200));
    }

    #[test]
    fn resolves_scalar_path_value() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![
                ScalarAnimationKey {
                    id: "a".to_string(),
                    time: at(0),
                    value: 0.0,
                    left_handle: None,
                    right_handle: None,
                    segment_to_next: ScalarSegmentType::Linear,
                    tangent_mode: TangentMode::Flat,
                },
                ScalarAnimationKey {
                    id: "b".to_string(),
                    time: at(100),
                    value: 1.0,
                    left_handle: None,
                    right_handle: None,
                    segment_to_next: ScalarSegmentType::Linear,
                    tangent_mode: TangentMode::Flat,
                },
            ],
            extrapolation: None,
        });
        let mut animations = BTreeMap::new();
        animations.insert("opacity".to_string(), ChannelData::Single(channel));
        assert_eq!(
            resolve_animation_path_value_at_time(
                Some(&animations),
                "opacity",
                at(50),
                &ParamValue::Number(1.0),
            ),
            ParamValue::Number(0.5)
        );
        assert_eq!(
            resolve_animation_path_value_at_time(
                Some(&animations),
                "volume",
                at(50),
                &ParamValue::Number(1.0),
            ),
            ParamValue::Number(1.0)
        );
    }

    #[test]
    fn resolves_composite_color_path_value() {
        let component = |value: f64| {
            AnimationChannel::Scalar(ScalarChannel {
                keys: vec![ScalarAnimationKey {
                    id: "k".to_string(),
                    time: at(0),
                    value,
                    left_handle: None,
                    right_handle: None,
                    segment_to_next: ScalarSegmentType::Linear,
                    tangent_mode: TangentMode::Flat,
                }],
                extrapolation: None,
            })
        };
        let mut components = BTreeMap::new();
        components.insert("r".to_string(), component(1.0));
        components.insert("g".to_string(), component(0.0));
        components.insert("b".to_string(), component(0.0));
        components.insert("a".to_string(), component(1.0));
        let mut animations = BTreeMap::new();
        animations.insert("color".to_string(), ChannelData::Composite(components));
        assert_eq!(
            resolve_animation_path_value_at_time(
                Some(&animations),
                "color",
                at(0),
                &ParamValue::String("#ffffff".to_string()),
            ),
            ParamValue::String("#ff0000".to_string())
        );
        assert_eq!(
            resolve_animation_path_value_at_time(
                Some(&animations),
                "color",
                at(0),
                &ParamValue::Number(0.0),
            ),
            ParamValue::Number(0.0)
        );
    }
}
