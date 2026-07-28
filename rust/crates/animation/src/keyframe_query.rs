use scene::{AnimationChannel, ChannelData, ElementAnimations, ParamValue};
use time::MediaTime;

use crate::channel_data::{get_channel_entries_from_data, is_animation_storage_key};
use crate::color::format_linear_rgba;
use crate::interpolation::{channel_value_at_time, scalar_segment_interpolation};
use crate::path::is_animation_path;
use crate::types::{AnimationInterpolation, ElementKeyframe};

#[derive(Clone, Debug)]
struct ChannelKeyframeMatch {
    component_index: usize,
    id: String,
    time: MediaTime,
    interpolation: AnimationInterpolation,
}

fn channel_keyframe_matches(data: Option<&ChannelData>) -> Vec<ChannelKeyframeMatch> {
    get_channel_entries_from_data(data)
        .into_iter()
        .enumerate()
        .flat_map(|(component_index, (_, channel))| {
            let matches: Vec<ChannelKeyframeMatch> = match channel {
                AnimationChannel::Scalar(scalar) => scalar
                    .keys
                    .iter()
                    .map(|key| ChannelKeyframeMatch {
                        component_index,
                        id: key.id.clone(),
                        time: key.time,
                        interpolation: scalar_segment_interpolation(key.segment_to_next),
                    })
                    .collect(),
                AnimationChannel::Discrete(discrete) => discrete
                    .keys
                    .iter()
                    .map(|key| ChannelKeyframeMatch {
                        component_index,
                        id: key.id.clone(),
                        time: key.time,
                        interpolation: AnimationInterpolation::Hold,
                    })
                    .collect(),
            };
            matches
        })
        .collect()
}

fn unique_channel_keyframe_matches(data: Option<&ChannelData>) -> Vec<ChannelKeyframeMatch> {
    let mut sorted = channel_keyframe_matches(data);
    sorted.sort_by(|left, right| {
        left.time
            .cmp(&right.time)
            .then(left.component_index.cmp(&right.component_index))
    });

    let mut unique: Vec<ChannelKeyframeMatch> = Vec::new();
    for keyframe_match in sorted {
        match unique.last() {
            Some(previous) if previous.time == keyframe_match.time => {
                if previous.component_index != 0 && keyframe_match.component_index == 0 {
                    *unique.last_mut().expect("last exists") = keyframe_match;
                }
            }
            _ => unique.push(keyframe_match),
        }
    }
    unique
}

fn preferred_channel_keyframe_match(
    matches: Vec<ChannelKeyframeMatch>,
) -> Option<ChannelKeyframeMatch> {
    matches
        .iter()
        .find(|keyframe_match| keyframe_match.component_index == 0)
        .cloned()
        .or_else(|| matches.into_iter().next())
}

fn channel_fallback_value(channel: &AnimationChannel) -> ParamValue {
    match channel {
        AnimationChannel::Scalar(scalar) => scalar
            .keys
            .first()
            .map(|key| ParamValue::Number(key.value))
            .unwrap_or(ParamValue::Number(0.0)),
        AnimationChannel::Discrete(discrete) => discrete
            .keys
            .first()
            .map(|key| key.value.clone().into())
            .unwrap_or(ParamValue::Bool(false)),
    }
}

fn channel_value(channel: &AnimationChannel, time: MediaTime) -> ParamValue {
    let fallback_value = channel_fallback_value(channel);
    channel_value_at_time(Some(channel), time, &fallback_value)
}

fn composed_channel_data_value_at_time(
    data: Option<&ChannelData>,
    time: MediaTime,
) -> Option<ParamValue> {
    let entries = get_channel_entries_from_data(data);
    if entries.is_empty() {
        return None;
    }
    if entries.len() == 1 && entries[0].0 == "value" {
        return Some(channel_value(entries[0].1, time));
    }

    let component_values: std::collections::BTreeMap<String, ParamValue> = entries
        .iter()
        .map(|(component_key, channel)| (component_key.clone(), channel_value(channel, time)))
        .collect();
    let (
        Some(ParamValue::Number(r)),
        Some(ParamValue::Number(g)),
        Some(ParamValue::Number(b)),
        Some(ParamValue::Number(a)),
    ) = (
        component_values.get("r"),
        component_values.get("g"),
        component_values.get("b"),
        component_values.get("a"),
    )
    else {
        return None;
    };
    Some(ParamValue::String(format_linear_rgba(
        &crate::color::LinearRgba {
            r: *r,
            g: *g,
            b: *b,
            a: *a,
        },
    )))
}

fn to_element_keyframe(
    data: Option<&ChannelData>,
    property_path: &str,
    keyframe_match: &ChannelKeyframeMatch,
) -> Option<ElementKeyframe> {
    let value = composed_channel_data_value_at_time(data, keyframe_match.time)?;
    Some(ElementKeyframe {
        property_path: property_path.to_string(),
        id: keyframe_match.id.clone(),
        time: keyframe_match.time,
        value,
        interpolation: keyframe_match.interpolation,
    })
}

pub fn get_element_keyframes(animations: Option<&ElementAnimations>) -> Vec<ElementKeyframe> {
    let Some(animations) = animations else {
        return Vec::new();
    };

    animations
        .iter()
        .filter(|(key, _)| is_animation_storage_key(key))
        .flat_map(|(property_path, data)| {
            if !is_animation_path(property_path) {
                return Vec::new();
            }
            unique_channel_keyframe_matches(Some(data))
                .iter()
                .filter_map(|keyframe_match| {
                    to_element_keyframe(Some(data), property_path, keyframe_match)
                })
                .collect()
        })
        .collect()
}

pub fn has_keyframes_for_path(
    animations: Option<&ElementAnimations>,
    property_path: &str,
) -> bool {
    get_channel_entries_from_data(animations.and_then(|a| a.get(property_path)))
        .iter()
        .any(|(_, channel)| {
            match channel {
                AnimationChannel::Scalar(scalar) => !scalar.keys.is_empty(),
                AnimationChannel::Discrete(discrete) => !discrete.keys.is_empty(),
            }
        })
}

pub fn get_keyframe_at_time(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    time: MediaTime,
) -> Option<ElementKeyframe> {
    let data = animations?.get(property_path)?;
    let keyframe_match = preferred_channel_keyframe_match(
        channel_keyframe_matches(Some(data))
            .into_iter()
            .filter(|keyframe_match| keyframe_match.time == time)
            .collect(),
    )?;
    to_element_keyframe(Some(data), property_path, &keyframe_match)
}

pub fn get_keyframe_by_id(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    keyframe_id: &str,
) -> Option<ElementKeyframe> {
    let data = animations?.get(property_path)?;
    let keyframe_match = preferred_channel_keyframe_match(
        channel_keyframe_matches(Some(data))
            .into_iter()
            .filter(|keyframe_match| keyframe_match.id == keyframe_id)
            .collect(),
    )?;
    to_element_keyframe(Some(data), property_path, &keyframe_match)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{ScalarAnimationKey, ScalarChannel, ScalarSegmentType, TangentMode};
    use std::collections::BTreeMap;

    fn key(id: &str, time: i64, value: f64) -> ScalarAnimationKey {
        ScalarAnimationKey {
            id: id.to_string(),
            time: MediaTime::from_ticks(time),
            value,
            left_handle: None,
            right_handle: None,
            segment_to_next: ScalarSegmentType::Linear,
            tangent_mode: TangentMode::Flat,
        }
    }

    fn animations_with_channel(path: &str, channel: AnimationChannel) -> ElementAnimations {
        let mut animations = BTreeMap::new();
        animations.insert(path.to_string(), ChannelData::Single(channel));
        animations
    }

    #[test]
    fn lists_keyframes_sorted_by_time() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![key("b", 100, 1.0), key("a", 0, 0.0)],
            extrapolation: None,
        });
        let animations = animations_with_channel("opacity", channel);
        let keyframes = get_element_keyframes(Some(&animations));
        assert_eq!(keyframes.len(), 2);
        assert_eq!(keyframes[0].id, "a");
        assert_eq!(keyframes[1].id, "b");
        assert_eq!(keyframes[0].value, ParamValue::Number(0.0));
        assert_eq!(keyframes[0].interpolation, AnimationInterpolation::Linear);
    }

    #[test]
    fn ignores_non_animation_and_legacy_paths() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![key("a", 0, 0.0)],
            extrapolation: None,
        });
        let animations = animations_with_channel("bindings", channel);
        assert!(get_element_keyframes(Some(&animations)).is_empty());
    }

    #[test]
    fn finds_keyframe_at_time_and_by_id() {
        let channel = AnimationChannel::Scalar(ScalarChannel {
            keys: vec![key("a", 0, 0.0), key("b", 100, 1.0)],
            extrapolation: None,
        });
        let animations = animations_with_channel("opacity", channel);
        let at_time = get_keyframe_at_time(Some(&animations), "opacity", MediaTime::from_ticks(100));
        assert_eq!(at_time.as_ref().map(|k| k.id.as_str()), Some("b"));
        assert!(get_keyframe_at_time(Some(&animations), "opacity", MediaTime::from_ticks(50)).is_none());
        let by_id = get_keyframe_by_id(Some(&animations), "opacity", "a");
        assert_eq!(by_id.as_ref().map(|k| k.time), Some(MediaTime::from_ticks(0)));
        assert!(has_keyframes_for_path(Some(&animations), "opacity"));
        assert!(!has_keyframes_for_path(Some(&animations), "volume"));
    }

    #[test]
    fn composite_color_data_composes_hex_value() {
        let component = |id: &str, time: i64, value: f64| {
            AnimationChannel::Scalar(ScalarChannel {
                keys: vec![key(id, time, value)],
                extrapolation: None,
            })
        };
        let mut components = BTreeMap::new();
        components.insert("r".to_string(), component("kr", 0, 1.0));
        components.insert("g".to_string(), component("kg", 0, 0.0));
        components.insert("b".to_string(), component("kb", 0, 0.0));
        components.insert("a".to_string(), component("ka", 0, 1.0));
        let mut animations = BTreeMap::new();
        animations.insert("color".to_string(), ChannelData::Composite(components));
        let keyframes = get_element_keyframes(Some(&animations));
        assert_eq!(keyframes.len(), 1);
        assert_eq!(keyframes[0].id, "kr");
        assert_eq!(keyframes[0].value, ParamValue::String("#ff0000".to_string()));
    }
}
