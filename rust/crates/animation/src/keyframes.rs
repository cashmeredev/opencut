use std::collections::{BTreeMap, HashMap};

use scene::{
    AnimationChannel, ChannelData, DiscreteAnimationKey, DiscreteChannel, DiscreteValue,
    ElementAnimations, ParamValue, ScalarAnimationKey, ScalarChannel, ScalarSegmentType,
    TangentMode,
};
use time::MediaTime;

use crate::bezier::{segment_handles, solve_bezier_progress_for_time};
use crate::channel_data::{
    get_channel_entries_from_data, get_channels_from_data, is_animation_storage_key,
};
use crate::channel_layout::{ChannelLayout, ChannelValueKind};
use crate::id::generate_uuid;
use crate::interpolation::{
    channel_value_at_time, is_scalar_channel, normalize_channel, normalize_discrete_channel,
    normalize_scalar_channel, scalar_segment_interpolation,
};
use crate::types::{AnimationInterpolation, ScalarCurveKeyframePatch};

fn round_media_time(time: f64) -> MediaTime {
    MediaTime::from_ticks(time.round() as i64)
}

fn has_channel_keys(channel: Option<&AnimationChannel>) -> bool {
    channel.is_some_and(|channel| !channel.keys().is_empty())
}

trait ChannelKeys {
    fn keys(&self) -> usize;
}

impl ChannelKeys for AnimationChannel {
    fn keys(&self) -> usize {
        match self {
            AnimationChannel::Scalar(scalar) => scalar.keys.len(),
            AnimationChannel::Discrete(discrete) => discrete.keys.len(),
        }
    }
}

fn has_channel_data(data: Option<&ChannelData>) -> bool {
    get_channels_from_data(data)
        .iter()
        .any(|channel| has_channel_keys(Some(channel)))
}

pub fn to_animation(animations: ElementAnimations) -> Option<ElementAnimations> {
    let next: ElementAnimations = animations
        .into_iter()
        .filter(|(key, data)| is_animation_storage_key(key) && has_channel_data(Some(data)))
        .collect();
    if next.is_empty() { None } else { Some(next) }
}

fn get_channel_from_data<'a>(
    data: Option<&'a ChannelData>,
    component_key: &str,
) -> Option<&'a AnimationChannel> {
    match data {
        Some(ChannelData::Single(channel)) => {
            if component_key == "value" { Some(channel) } else { None }
        }
        Some(ChannelData::Composite(components)) => components.get(component_key),
        None => None,
    }
}

fn get_primary_component_key(channel_layout: &ChannelLayout) -> String {
    channel_layout
        .components()
        .first()
        .map(|component| component.key.clone())
        .unwrap_or_else(|| "value".to_string())
}

fn get_primary_channel_from_data<'a>(
    data: Option<&'a ChannelData>,
    channel_layout: &ChannelLayout,
) -> Option<&'a AnimationChannel> {
    get_channel_from_data(data, &get_primary_component_key(channel_layout))
}

fn set_channel_in_data(
    data: Option<ChannelData>,
    component_key: &str,
    channel: Option<AnimationChannel>,
) -> Option<ChannelData> {
    if component_key == "value" {
        return channel.map(ChannelData::Single);
    }

    let mut components = match data {
        Some(ChannelData::Composite(components)) => components,
        _ => BTreeMap::new(),
    };
    match channel {
        Some(channel) if has_channel_keys(Some(&channel)) => {
            components.insert(component_key.to_string(), channel);
        }
        _ => {
            components.remove(component_key);
        }
    }
    if components.is_empty() {
        None
    } else {
        Some(ChannelData::Composite(components))
    }
}

fn scalar_segment_type(interpolation: AnimationInterpolation) -> ScalarSegmentType {
    match interpolation {
        AnimationInterpolation::Hold => ScalarSegmentType::Step,
        AnimationInterpolation::Bezier => ScalarSegmentType::Bezier,
        AnimationInterpolation::Linear => ScalarSegmentType::Linear,
    }
}

fn create_scalar_key(
    id: String,
    time: MediaTime,
    value: f64,
    interpolation: Option<AnimationInterpolation>,
    previous_key: Option<&ScalarAnimationKey>,
) -> ScalarAnimationKey {
    ScalarAnimationKey {
        id,
        time,
        value,
        left_handle: previous_key.and_then(|key| key.left_handle),
        right_handle: previous_key.and_then(|key| key.right_handle),
        segment_to_next: previous_key.map(|key| key.segment_to_next).unwrap_or_else(|| {
            scalar_segment_type(interpolation.unwrap_or(AnimationInterpolation::Linear))
        }),
        tangent_mode: previous_key
            .map(|key| key.tangent_mode)
            .unwrap_or(TangentMode::Flat),
    }
}

fn channel_key_ids_and_times(channel: &AnimationChannel) -> Vec<(String, MediaTime)> {
    match channel {
        AnimationChannel::Scalar(scalar) => scalar
            .keys
            .iter()
            .map(|key| (key.id.clone(), key.time))
            .collect(),
        AnimationChannel::Discrete(discrete) => discrete
            .keys
            .iter()
            .map(|key| (key.id.clone(), key.time))
            .collect(),
    }
}

fn get_target_key_metadata(
    channel: Option<&AnimationChannel>,
    time: MediaTime,
    keyframe_id: Option<&str>,
) -> (String, MediaTime) {
    let normalized = channel.map(normalize_channel);
    let keys = normalized
        .as_ref()
        .map(channel_key_ids_and_times)
        .unwrap_or_default();

    if let Some(id) = keyframe_id {
        if let Some((found_id, _)) = keys.iter().find(|(key_id, _)| key_id == id) {
            return (found_id.clone(), time);
        }
    }

    if let Some((found_id, found_time)) = keys.iter().find(|(_, key_time)| *key_time == time) {
        return (found_id.clone(), *found_time);
    }

    (
        keyframe_id.map(str::to_string).unwrap_or_else(generate_uuid),
        time,
    )
}

pub fn upsert_discrete_channel_key(
    channel: Option<&DiscreteChannel>,
    time: MediaTime,
    value: DiscreteValue,
    keyframe_id: Option<&str>,
) -> DiscreteChannel {
    let normalized = normalize_discrete_channel(&channel.cloned().unwrap_or(DiscreteChannel {
        keys: Vec::new(),
    }));
    let mut keys = normalized.keys;

    if let Some(id) = keyframe_id {
        if let Some(index) = keys.iter().position(|key| key.id == id) {
            keys[index] = DiscreteAnimationKey {
                id: keys[index].id.clone(),
                time,
                value,
            };
            return normalize_discrete_channel(&DiscreteChannel { keys });
        }
    }

    if let Some(index) = keys.iter().position(|key| key.time == time) {
        keys[index] = DiscreteAnimationKey {
            id: keys[index].id.clone(),
            time: keys[index].time,
            value,
        };
        return normalize_discrete_channel(&DiscreteChannel { keys });
    }

    keys.push(DiscreteAnimationKey {
        id: keyframe_id.map(str::to_string).unwrap_or_else(generate_uuid),
        time,
        value,
    });
    normalize_discrete_channel(&DiscreteChannel { keys })
}

fn replace_scalar_key_at(
    keys: &mut [ScalarAnimationKey],
    index: usize,
    time: MediaTime,
    value: f64,
    interpolation: Option<AnimationInterpolation>,
    keep_existing_time: bool,
) {
    let mut previous_key = keys[index].clone();
    if let Some(interpolation) = interpolation {
        previous_key.segment_to_next = scalar_segment_type(interpolation);
    }
    let key_time = if keep_existing_time {
        previous_key.time
    } else {
        time
    };
    keys[index] = create_scalar_key(
        previous_key.id.clone(),
        key_time,
        value,
        interpolation,
        Some(&previous_key),
    );
}

pub fn upsert_scalar_channel_key(
    channel: Option<&ScalarChannel>,
    time: MediaTime,
    value: f64,
    interpolation: Option<AnimationInterpolation>,
    default_interpolation: Option<AnimationInterpolation>,
    keyframe_id: Option<&str>,
) -> ScalarChannel {
    let normalized = normalize_scalar_channel(&channel.cloned().unwrap_or(ScalarChannel {
        keys: Vec::new(),
        extrapolation: None,
    }));
    let mut keys = normalized.keys;

    if let Some(id) = keyframe_id {
        if let Some(index) = keys.iter().position(|key| key.id == id) {
            replace_scalar_key_at(&mut keys, index, time, value, interpolation, false);
            return normalize_scalar_channel(&ScalarChannel {
                keys,
                extrapolation: normalized.extrapolation,
            });
        }
    }

    if let Some(index) = keys.iter().position(|key| key.time == time) {
        replace_scalar_key_at(&mut keys, index, time, value, interpolation, true);
        return normalize_scalar_channel(&ScalarChannel {
            keys,
            extrapolation: normalized.extrapolation,
        });
    }

    keys.push(create_scalar_key(
        keyframe_id.map(str::to_string).unwrap_or_else(generate_uuid),
        time,
        value,
        interpolation.or(default_interpolation),
        None,
    ));
    normalize_scalar_channel(&ScalarChannel {
        keys,
        extrapolation: normalized.extrapolation,
    })
}

pub fn get_channel<'a>(
    animations: Option<&'a ElementAnimations>,
    property_path: &str,
) -> Option<&'a AnimationChannel> {
    let data = animations?.get(property_path);
    match data {
        ChannelData::Single(channel) => Some(channel),
        ChannelData::Composite(_) => get_channels_from_data(Some(data)).into_iter().next(),
    }
}

pub fn upsert_path_keyframe(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    time: MediaTime,
    value: &ParamValue,
    interpolation: Option<AnimationInterpolation>,
    keyframe_id: Option<&str>,
    channel_layout: &ChannelLayout,
    coerce_value: impl Fn(&ParamValue) -> Option<ParamValue>,
) -> Option<ElementAnimations> {
    let coerced_value = coerce_value(value)?;

    let mut next_animations = animations.cloned().unwrap_or_default();
    let current_data = animations.and_then(|a| a.get(property_path)).cloned();
    let primary_channel = get_primary_channel_from_data(current_data.as_ref(), channel_layout);
    let (target_id, target_time) = get_target_key_metadata(primary_channel, time, keyframe_id);
    let component_values = channel_layout.decompose(&coerced_value)?;

    let mut next_data = current_data.clone();
    for component in channel_layout.components() {
        let component_key = component.key.as_str();
        let Some(next_value) = component_values.get(component_key) else {
            continue;
        };
        let current_channel = get_channel_from_data(current_data.as_ref(), component_key);

        match component.value_kind {
            ChannelValueKind::Discrete => {
                let discrete_value = match next_value {
                    ParamValue::String(value) => DiscreteValue::String(value.clone()),
                    ParamValue::Bool(value) => DiscreteValue::Bool(*value),
                    ParamValue::Number(_) => continue,
                };
                let discrete_channel = match current_channel {
                    Some(AnimationChannel::Discrete(discrete)) => Some(discrete),
                    _ => None,
                };
                let next_channel = upsert_discrete_channel_key(
                    discrete_channel,
                    target_time,
                    discrete_value,
                    Some(&target_id),
                );
                next_data = set_channel_in_data(
                    next_data,
                    component_key,
                    Some(AnimationChannel::Discrete(next_channel)),
                );
            }
            ChannelValueKind::Scalar => {
                let ParamValue::Number(number_value) = next_value else {
                    continue;
                };
                let scalar_channel = match current_channel {
                    Some(AnimationChannel::Scalar(scalar)) => Some(scalar),
                    _ => None,
                };
                let next_channel = upsert_scalar_channel_key(
                    scalar_channel,
                    target_time,
                    *number_value,
                    interpolation,
                    Some(component.default_interpolation),
                    Some(&target_id),
                );
                next_data = set_channel_in_data(
                    next_data,
                    component_key,
                    Some(AnimationChannel::Scalar(next_channel)),
                );
            }
        }
    }

    match next_data {
        Some(data) => {
            next_animations.insert(property_path.to_string(), data);
        }
        None => {
            next_animations.remove(property_path);
        }
    }
    to_animation(next_animations)
}

pub fn upsert_keyframe(
    channel: Option<&AnimationChannel>,
    time: MediaTime,
    value: &ParamValue,
    interpolation: Option<AnimationInterpolation>,
    keyframe_id: Option<&str>,
) -> Option<AnimationChannel> {
    let channel = channel?;

    match value {
        ParamValue::String(value) => {
            let discrete = match channel {
                AnimationChannel::Discrete(discrete) => Some(discrete),
                _ => None,
            };
            Some(AnimationChannel::Discrete(upsert_discrete_channel_key(
                discrete,
                time,
                DiscreteValue::String(value.clone()),
                keyframe_id,
            )))
        }
        ParamValue::Bool(value) => {
            let discrete = match channel {
                AnimationChannel::Discrete(discrete) => Some(discrete),
                _ => None,
            };
            Some(AnimationChannel::Discrete(upsert_discrete_channel_key(
                discrete,
                time,
                DiscreteValue::Bool(*value),
                keyframe_id,
            )))
        }
        ParamValue::Number(value) => {
            let scalar = match channel {
                AnimationChannel::Scalar(scalar) => Some(scalar),
                _ => None,
            };
            Some(AnimationChannel::Scalar(upsert_scalar_channel_key(
                scalar,
                time,
                *value,
                interpolation,
                None,
                keyframe_id,
            )))
        }
    }
}

pub fn remove_keyframe(
    channel: Option<&AnimationChannel>,
    keyframe_id: &str,
) -> Option<AnimationChannel> {
    let channel = channel?;

    match channel {
        AnimationChannel::Scalar(scalar) => {
            let keys: Vec<ScalarAnimationKey> = scalar
                .keys
                .iter()
                .filter(|key| key.id != keyframe_id)
                .cloned()
                .collect();
            if keys.is_empty() {
                return None;
            }
            Some(AnimationChannel::Scalar(normalize_scalar_channel(
                &ScalarChannel {
                    keys,
                    extrapolation: scalar.extrapolation.clone(),
                },
            )))
        }
        AnimationChannel::Discrete(discrete) => {
            let keys: Vec<DiscreteAnimationKey> = discrete
                .keys
                .iter()
                .filter(|key| key.id != keyframe_id)
                .cloned()
                .collect();
            if keys.is_empty() {
                return None;
            }
            Some(AnimationChannel::Discrete(normalize_discrete_channel(
                &DiscreteChannel { keys },
            )))
        }
    }
}

pub fn retime_keyframe(
    channel: Option<&AnimationChannel>,
    keyframe_id: &str,
    time: MediaTime,
) -> Option<AnimationChannel> {
    let channel = channel?;

    match channel {
        AnimationChannel::Scalar(scalar) => {
            let Some(index) = scalar.keys.iter().position(|key| key.id == keyframe_id) else {
                return Some(channel.clone());
            };
            let mut keys = scalar.keys.clone();
            keys[index].time = time;
            Some(AnimationChannel::Scalar(normalize_scalar_channel(
                &ScalarChannel {
                    keys,
                    extrapolation: scalar.extrapolation.clone(),
                },
            )))
        }
        AnimationChannel::Discrete(discrete) => {
            let Some(index) = discrete.keys.iter().position(|key| key.id == keyframe_id) else {
                return Some(channel.clone());
            };
            let mut keys = discrete.keys.clone();
            keys[index].time = time;
            Some(AnimationChannel::Discrete(normalize_discrete_channel(
                &DiscreteChannel { keys },
            )))
        }
    }
}

pub fn set_channel(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    channel: Option<&AnimationChannel>,
) -> Option<ElementAnimations> {
    set_binding_component_channel(animations, property_path, "value", channel)
}

pub fn set_binding_component_channel(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    component_key: &str,
    channel: Option<&AnimationChannel>,
) -> Option<ElementAnimations> {
    let mut next_animations = animations.cloned().unwrap_or_default();
    let prepared = channel
        .filter(|channel| has_channel_keys(Some(channel)))
        .map(normalize_channel);
    let data = set_channel_in_data(
        next_animations.get(property_path).cloned(),
        component_key,
        prepared,
    );
    match data {
        Some(data) => {
            next_animations.insert(property_path.to_string(), data);
        }
        None => {
            next_animations.remove(property_path);
        }
    }
    to_animation(next_animations)
}

pub fn update_scalar_keyframe_curve(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    component_key: &str,
    keyframe_id: &str,
    patch: &ScalarCurveKeyframePatch,
) -> Option<ElementAnimations> {
    let channel = get_channel_from_data(
        animations.and_then(|a| a.get(property_path)),
        component_key,
    );
    let Some(AnimationChannel::Scalar(scalar)) = channel else {
        return animations.cloned();
    };

    let Some(keyframe_index) = scalar.keys.iter().position(|key| key.id == keyframe_id) else {
        return animations.cloned();
    };

    let mut keys = scalar.keys.clone();
    let current_key = &keys[keyframe_index];
    keys[keyframe_index] = ScalarAnimationKey {
        left_handle: patch.left_handle.unwrap_or(current_key.left_handle),
        right_handle: patch.right_handle.unwrap_or(current_key.right_handle),
        segment_to_next: patch.segment_to_next.unwrap_or(current_key.segment_to_next),
        tangent_mode: patch.tangent_mode.unwrap_or(current_key.tangent_mode),
        ..current_key.clone()
    };

    set_binding_component_channel(
        animations,
        property_path,
        component_key,
        Some(&AnimationChannel::Scalar(ScalarChannel {
            keys,
            extrapolation: scalar.extrapolation.clone(),
        })),
    )
}

fn clone_channel_with_key_ids(
    channel: &AnimationChannel,
    key_id_map: &HashMap<String, String>,
) -> AnimationChannel {
    let remap = |id: &str| key_id_map.get(id).cloned().unwrap_or_else(|| id.to_string());
    match channel {
        AnimationChannel::Scalar(scalar) => AnimationChannel::Scalar(normalize_scalar_channel(
            &ScalarChannel {
                keys: scalar
                    .keys
                    .iter()
                    .map(|key| ScalarAnimationKey {
                        id: remap(&key.id),
                        ..key.clone()
                    })
                    .collect(),
                extrapolation: scalar.extrapolation.clone(),
            },
        )),
        AnimationChannel::Discrete(discrete) => {
            AnimationChannel::Discrete(normalize_discrete_channel(&DiscreteChannel {
                keys: discrete
                    .keys
                    .iter()
                    .map(|key| DiscreteAnimationKey {
                        id: remap(&key.id),
                        ..key.clone()
                    })
                    .collect(),
            }))
        }
    }
}

pub fn clone_animations(
    animations: Option<&ElementAnimations>,
    regenerate_keyframe_ids: bool,
) -> Option<ElementAnimations> {
    let animations = animations?;

    let mut next_animations = animations.clone();
    for (property_path, data) in animations
        .iter()
        .filter(|(key, _)| is_animation_storage_key(key))
    {
        let channels = get_channels_from_data(Some(data));
        let mut key_id_map = HashMap::new();
        if let Some(primary_channel) = channels.first() {
            for (id, _) in channel_key_ids_and_times(primary_channel) {
                let next_id = if regenerate_keyframe_ids {
                    generate_uuid()
                } else {
                    id.clone()
                };
                key_id_map.insert(id, next_id);
            }
        }

        let next_data = match data {
            ChannelData::Single(channel) => {
                ChannelData::Single(clone_channel_with_key_ids(channel, &key_id_map))
            }
            ChannelData::Composite(components) => ChannelData::Composite(
                components
                    .iter()
                    .map(|(component_key, channel)| {
                        (
                            component_key.clone(),
                            clone_channel_with_key_ids(channel, &key_id_map),
                        )
                    })
                    .collect(),
            ),
        };
        next_animations.insert(property_path.clone(), next_data);
    }

    to_animation(next_animations)
}

pub fn clamp_animations_to_duration(
    animations: Option<&ElementAnimations>,
    duration: MediaTime,
) -> Option<ElementAnimations> {
    let animations = animations?;
    if duration <= MediaTime::ZERO {
        return None;
    }
    split_animations_at_time_with_options(Some(animations), duration, true).0
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn lerp_point(left: Point, right: Point, progress: f64) -> Point {
    Point {
        x: left.x + (right.x - left.x) * progress,
        y: left.y + (right.y - left.y) * progress,
    }
}

pub struct SplitChannelResult {
    pub left_channel: Option<AnimationChannel>,
    pub right_channel: Option<AnimationChannel>,
}

fn split_discrete_channel_at_time(
    channel: Option<&DiscreteChannel>,
    split_time: MediaTime,
    left_boundary_id: &str,
    right_boundary_id: &str,
    include_split_boundary: bool,
) -> SplitChannelResult {
    let Some(channel) = channel.filter(|channel| !channel.keys.is_empty()) else {
        return SplitChannelResult {
            left_channel: None,
            right_channel: None,
        };
    };

    let normalized = normalize_discrete_channel(channel);
    let mut left_keys: Vec<DiscreteAnimationKey> = normalized
        .keys
        .iter()
        .filter(|key| key.time <= split_time)
        .cloned()
        .collect();
    let mut right_keys: Vec<DiscreteAnimationKey> = normalized
        .keys
        .iter()
        .filter(|key| key.time >= split_time)
        .map(|key| DiscreteAnimationKey {
            time: key.time - split_time,
            ..key.clone()
        })
        .collect();

    if include_split_boundary {
        let has_boundary_on_left = left_keys.iter().any(|key| key.time == split_time);
        let has_boundary_on_right = right_keys.iter().any(|key| key.time == MediaTime::ZERO);
        let boundary_value = discrete_value_at_time(
            &normalized,
            split_time,
            normalized.keys[0].value.clone(),
        );
        if !has_boundary_on_left {
            left_keys.push(DiscreteAnimationKey {
                id: left_boundary_id.to_string(),
                time: split_time,
                value: boundary_value.clone(),
            });
        }
        if !has_boundary_on_right {
            right_keys.insert(
                0,
                DiscreteAnimationKey {
                    id: right_boundary_id.to_string(),
                    time: MediaTime::ZERO,
                    value: boundary_value,
                },
            );
        }
    }

    SplitChannelResult {
        left_channel: if left_keys.is_empty() {
            None
        } else {
            Some(AnimationChannel::Discrete(normalize_discrete_channel(
                &DiscreteChannel { keys: left_keys },
            )))
        },
        right_channel: if right_keys.is_empty() {
            None
        } else {
            Some(AnimationChannel::Discrete(normalize_discrete_channel(
                &DiscreteChannel { keys: right_keys },
            )))
        },
    }
}

fn discrete_value_at_time(
    channel: &DiscreteChannel,
    time: MediaTime,
    fallback_value: DiscreteValue,
) -> DiscreteValue {
    let ParamValue::Bool(_) = ParamValue::Bool(false) else {
        unreachable!()
    };
    crate::interpolation::discrete_channel_value_at_time(Some(channel), time, fallback_value)
}

fn build_split_scalar_result(
    left_keys: Vec<ScalarAnimationKey>,
    right_keys: Vec<ScalarAnimationKey>,
    extrapolation: Option<scene::ChannelExtrapolation>,
) -> SplitChannelResult {
    SplitChannelResult {
        left_channel: if left_keys.is_empty() {
            None
        } else {
            Some(AnimationChannel::Scalar(normalize_scalar_channel(
                &ScalarChannel {
                    keys: left_keys,
                    extrapolation: extrapolation.clone(),
                },
            )))
        },
        right_channel: if right_keys.is_empty() {
            None
        } else {
            Some(AnimationChannel::Scalar(normalize_scalar_channel(
                &ScalarChannel {
                    keys: right_keys,
                    extrapolation,
                },
            )))
        },
    }
}

fn split_scalar_channel_at_time(
    channel: Option<&ScalarChannel>,
    split_time: MediaTime,
    left_boundary_id: &str,
    right_boundary_id: &str,
    include_split_boundary: bool,
) -> SplitChannelResult {
    let Some(channel) = channel.filter(|channel| !channel.keys.is_empty()) else {
        return SplitChannelResult {
            left_channel: None,
            right_channel: None,
        };
    };

    let normalized = normalize_scalar_channel(channel);
    let mut left_keys: Vec<ScalarAnimationKey> = normalized
        .keys
        .iter()
        .filter(|key| key.time <= split_time)
        .cloned()
        .collect();
    let mut right_keys: Vec<ScalarAnimationKey> = normalized
        .keys
        .iter()
        .filter(|key| key.time >= split_time)
        .map(|key| ScalarAnimationKey {
            time: key.time - split_time,
            ..key.clone()
        })
        .collect();

    let has_boundary_on_left = left_keys.iter().any(|key| key.time == split_time);
    let has_boundary_on_right = right_keys.iter().any(|key| key.time == MediaTime::ZERO);
    if !include_split_boundary || (has_boundary_on_left && has_boundary_on_right) {
        return build_split_scalar_result(
            left_keys,
            right_keys,
            normalized.extrapolation.clone(),
        );
    }

    for key_index in 0..normalized.keys.len().saturating_sub(1) {
        let left_key = &normalized.keys[key_index];
        let right_key = &normalized.keys[key_index + 1];
        if split_time <= left_key.time || split_time >= right_key.time {
            continue;
        }

        let boundary_value = crate::interpolation::scalar_channel_value_at_time(
            Some(&normalized),
            split_time,
            left_key.value,
        );

        if left_key.segment_to_next == ScalarSegmentType::Bezier {
            let (right_handle, left_handle) = segment_handles(left_key, right_key);
            let progress = solve_bezier_progress_for_time(split_time, left_key, right_key);
            let p0 = Point {
                x: left_key.time.as_ticks() as f64,
                y: left_key.value,
            };
            let p1 = Point {
                x: p0.x + right_handle.dt,
                y: p0.y + right_handle.dv,
            };
            let p3 = Point {
                x: right_key.time.as_ticks() as f64,
                y: right_key.value,
            };
            let p2 = Point {
                x: p3.x + left_handle.dt,
                y: p3.y + left_handle.dv,
            };
            let q0 = lerp_point(p0, p1, progress);
            let q1 = lerp_point(p1, p2, progress);
            let q2 = lerp_point(p2, p3, progress);
            let r0 = lerp_point(q0, q1, progress);
            let r1 = lerp_point(q1, q2, progress);
            let split_point = lerp_point(r0, r1, progress);

            let mut next_left_keys: Vec<ScalarAnimationKey> = normalized
                .keys
                .iter()
                .filter(|key| key.time < split_time)
                .cloned()
                .collect();
            next_left_keys.push(ScalarAnimationKey {
                right_handle: Some(scene::CurveHandle {
                    dt: round_media_time(q0.x - p0.x),
                    dv: q0.y - p0.y,
                }),
                ..left_key.clone()
            });
            next_left_keys.push(ScalarAnimationKey {
                id: left_boundary_id.to_string(),
                time: split_time,
                value: boundary_value,
                left_handle: Some(scene::CurveHandle {
                    dt: round_media_time(r0.x - split_point.x),
                    dv: r0.y - split_point.y,
                }),
                right_handle: None,
                segment_to_next: left_key.segment_to_next,
                tangent_mode: left_key.tangent_mode,
            });

            let mut next_right_keys: Vec<ScalarAnimationKey> = vec![
                ScalarAnimationKey {
                    id: right_boundary_id.to_string(),
                    time: MediaTime::ZERO,
                    value: boundary_value,
                    left_handle: None,
                    right_handle: Some(scene::CurveHandle {
                        dt: round_media_time(r1.x - split_point.x),
                        dv: r1.y - split_point.y,
                    }),
                    segment_to_next: ScalarSegmentType::Bezier,
                    tangent_mode: left_key.tangent_mode,
                },
                ScalarAnimationKey {
                    time: right_key.time - split_time,
                    left_handle: Some(scene::CurveHandle {
                        dt: round_media_time(q2.x - p3.x),
                        dv: q2.y - p3.y,
                    }),
                    ..right_key.clone()
                },
            ];
            next_right_keys.extend(
                normalized
                    .keys
                    .iter()
                    .filter(|key| key.time > right_key.time)
                    .map(|key| ScalarAnimationKey {
                        time: key.time - split_time,
                        ..key.clone()
                    }),
            );

            return SplitChannelResult {
                left_channel: Some(AnimationChannel::Scalar(normalize_scalar_channel(
                    &ScalarChannel {
                        keys: next_left_keys,
                        extrapolation: normalized.extrapolation.clone(),
                    },
                ))),
                right_channel: Some(AnimationChannel::Scalar(normalize_scalar_channel(
                    &ScalarChannel {
                        keys: next_right_keys,
                        extrapolation: normalized.extrapolation.clone(),
                    },
                ))),
            };
        }

        left_keys.push(create_scalar_key(
            left_boundary_id.to_string(),
            split_time,
            boundary_value,
            Some(AnimationInterpolation::Linear),
            None,
        ));
        right_keys.insert(
            0,
            create_scalar_key(
                right_boundary_id.to_string(),
                MediaTime::ZERO,
                boundary_value,
                Some(scalar_segment_interpolation(left_key.segment_to_next)),
                None,
            ),
        );
        return build_split_scalar_result(
            left_keys,
            right_keys,
            normalized.extrapolation.clone(),
        );
    }

    build_split_scalar_result(left_keys, right_keys, normalized.extrapolation.clone())
}

fn split_channel_at_time(
    channel: Option<&AnimationChannel>,
    split_time: MediaTime,
    left_boundary_id: &str,
    right_boundary_id: &str,
    include_split_boundary: bool,
) -> SplitChannelResult {
    match channel {
        Some(AnimationChannel::Discrete(discrete)) => split_discrete_channel_at_time(
            Some(discrete),
            split_time,
            left_boundary_id,
            right_boundary_id,
            include_split_boundary,
        ),
        scalar => split_scalar_channel_at_time(
            match scalar {
                Some(AnimationChannel::Scalar(scalar)) => Some(scalar),
                _ => None,
            },
            split_time,
            left_boundary_id,
            right_boundary_id,
            include_split_boundary,
        ),
    }
}

pub fn split_animations_at_time_with_options(
    animations: Option<&ElementAnimations>,
    split_time: MediaTime,
    include_split_boundary: bool,
) -> (Option<ElementAnimations>, Option<ElementAnimations>) {
    let Some(animations) = animations else {
        return (None, None);
    };

    let mut left_animations = ElementAnimations::new();
    let mut right_animations = ElementAnimations::new();

    for (property_path, data) in animations
        .iter()
        .filter(|(key, _)| is_animation_storage_key(key))
    {
        let left_boundary_id = generate_uuid();
        let right_boundary_id = generate_uuid();

        for (component_key, channel) in get_channel_entries_from_data(Some(data)) {
            let split = split_channel_at_time(
                Some(channel),
                split_time,
                &left_boundary_id,
                &right_boundary_id,
                include_split_boundary,
            );
            if let Some(left_channel) = split.left_channel {
                if let Some(data) = set_channel_in_data(
                    left_animations.get(property_path).cloned(),
                    &component_key,
                    Some(left_channel),
                ) {
                    left_animations.insert(property_path.clone(), data);
                }
            }
            if let Some(right_channel) = split.right_channel {
                if let Some(data) = set_channel_in_data(
                    right_animations.get(property_path).cloned(),
                    &component_key,
                    Some(right_channel),
                ) {
                    right_animations.insert(property_path.clone(), data);
                }
            }
        }
    }

    (to_animation(left_animations), to_animation(right_animations))
}

pub fn split_animations_at_time(
    animations: &ElementAnimations,
    split_time: MediaTime,
) -> (ElementAnimations, ElementAnimations) {
    let (left, right) = split_animations_at_time_with_options(Some(animations), split_time, true);
    (left.unwrap_or_default(), right.unwrap_or_default())
}

pub fn remove_element_keyframe(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    keyframe_id: &str,
) -> Option<ElementAnimations> {
    let data = animations.and_then(|a| a.get(property_path))?;

    let mut next_animations = animations.cloned().unwrap_or_default();
    let next_data = match data {
        ChannelData::Single(channel) => remove_keyframe(Some(channel), keyframe_id),
        ChannelData::Composite(components) => {
            let mut next_data = Some(data.clone());
            for (component_key, channel) in components {
                next_data = set_channel_in_data(
                    next_data,
                    component_key,
                    remove_keyframe(Some(channel), keyframe_id),
                );
            }
            next_data
        }
    };
    match next_data {
        Some(data) => {
            next_animations.insert(property_path.to_string(), data);
        }
        None => {
            next_animations.remove(property_path);
        }
    }
    to_animation(next_animations)
}

pub fn retime_element_keyframe(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    keyframe_id: &str,
    time: MediaTime,
) -> Option<ElementAnimations> {
    let data = animations.and_then(|a| a.get(property_path))?;

    let mut next_animations = animations.cloned().unwrap_or_default();
    let next_data = match data {
        ChannelData::Single(channel) => retime_keyframe(Some(channel), keyframe_id, time),
        ChannelData::Composite(components) => {
            let mut next_data = Some(data.clone());
            for (component_key, channel) in components {
                next_data = set_channel_in_data(
                    next_data,
                    component_key,
                    retime_keyframe(Some(channel), keyframe_id, time),
                );
            }
            next_data
        }
    };
    match next_data {
        Some(data) => {
            next_animations.insert(property_path.to_string(), data);
        }
        None => {
            next_animations.remove(property_path);
        }
    }
    to_animation(next_animations)
}
