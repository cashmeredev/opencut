use scene::{
    AnimationChannel, ChannelExtrapolationMode, CurveHandle, DiscreteChannel, DiscreteValue,
    ParamValue, ScalarAnimationKey, ScalarChannel, ScalarSegmentType,
};
use time::MediaTime;

use crate::bezier::{bezier_point, segment_handles, solve_bezier_progress_for_time};
use crate::types::AnimationInterpolation;

pub fn is_scalar_channel(channel: &AnimationChannel) -> bool {
    matches!(channel, AnimationChannel::Scalar(_))
}

fn normalize_right_handle(
    handle: Option<CurveHandle>,
    left_key: &ScalarAnimationKey,
    right_key: &ScalarAnimationKey,
) -> Option<CurveHandle> {
    let handle = handle?;
    let span = (right_key.time - left_key.time).as_ticks().max(1);
    Some(CurveHandle {
        dt: MediaTime::from_ticks(handle.dt.as_ticks().clamp(0, span)),
        dv: handle.dv,
    })
}

fn normalize_left_handle(
    handle: Option<CurveHandle>,
    left_key: &ScalarAnimationKey,
    right_key: &ScalarAnimationKey,
) -> Option<CurveHandle> {
    let handle = handle?;
    let span = (right_key.time - left_key.time).as_ticks().max(1);
    Some(CurveHandle {
        dt: MediaTime::from_ticks(handle.dt.as_ticks().clamp(-span, 0)),
        dv: handle.dv,
    })
}

pub fn normalize_scalar_channel(channel: &ScalarChannel) -> ScalarChannel {
    let mut sorted_keys = channel.keys.clone();
    sorted_keys.sort_by_key(|key| key.time);

    let mut next_keys = Vec::with_capacity(sorted_keys.len());
    for (index, key) in sorted_keys.iter().enumerate() {
        let previous_key = index.checked_sub(1).and_then(|i| sorted_keys.get(i));
        let next_key = sorted_keys.get(index + 1);
        let left_handle =
            previous_key.and_then(|previous| normalize_left_handle(key.left_handle, previous, key));
        let right_handle =
            next_key.and_then(|next| normalize_right_handle(key.right_handle, key, next));
        next_keys.push(ScalarAnimationKey {
            left_handle,
            right_handle,
            ..key.clone()
        });
    }

    ScalarChannel {
        keys: next_keys,
        extrapolation: channel.extrapolation.clone(),
    }
}

pub fn normalize_discrete_channel(channel: &DiscreteChannel) -> DiscreteChannel {
    let mut keys = channel.keys.clone();
    keys.sort_by_key(|key| key.time);
    DiscreteChannel { keys }
}

pub fn normalize_channel(channel: &AnimationChannel) -> AnimationChannel {
    match channel {
        AnimationChannel::Scalar(scalar) => {
            AnimationChannel::Scalar(normalize_scalar_channel(scalar))
        }
        AnimationChannel::Discrete(discrete) => {
            AnimationChannel::Discrete(normalize_discrete_channel(discrete))
        }
    }
}

pub fn scalar_segment_interpolation(segment: ScalarSegmentType) -> AnimationInterpolation {
    match segment {
        ScalarSegmentType::Step => AnimationInterpolation::Hold,
        ScalarSegmentType::Linear => AnimationInterpolation::Linear,
        ScalarSegmentType::Bezier => AnimationInterpolation::Bezier,
    }
}

fn extrapolate_scalar_edge(
    mode: ChannelExtrapolationMode,
    edge_key: &ScalarAnimationKey,
    neighbor_key: Option<&ScalarAnimationKey>,
    time: f64,
) -> f64 {
    let Some(neighbor_key) = neighbor_key else {
        return edge_key.value;
    };
    if mode == ChannelExtrapolationMode::Hold {
        return edge_key.value;
    }

    let span = (neighbor_key.time - edge_key.time).as_ticks() as f64;
    if span == 0.0 {
        return edge_key.value;
    }

    edge_key.value
        + ((time - edge_key.time.as_ticks() as f64) / span) * (neighbor_key.value - edge_key.value)
}

fn lerp_number(left_value: f64, right_value: f64, progress: f64) -> f64 {
    left_value + (right_value - left_value) * progress
}

pub fn scalar_channel_value_at_time(
    channel: Option<&ScalarChannel>,
    time: MediaTime,
    fallback_value: f64,
) -> f64 {
    let Some(channel) = channel.filter(|channel| !channel.keys.is_empty()) else {
        return fallback_value;
    };

    let normalized = normalize_scalar_channel(channel);
    let keys = &normalized.keys;
    let first_key = &keys[0];
    let last_key = &keys[keys.len() - 1];
    let t = time.as_ticks() as f64;

    if time <= first_key.time {
        if time < first_key.time {
            return extrapolate_scalar_edge(
                normalized
                    .extrapolation
                    .as_ref()
                    .map(|e| e.before)
                    .unwrap_or(ChannelExtrapolationMode::Hold),
                first_key,
                keys.get(1),
                t,
            );
        }
        return first_key.value;
    }

    if time >= last_key.time {
        if time > last_key.time {
            return extrapolate_scalar_edge(
                normalized
                    .extrapolation
                    .as_ref()
                    .map(|e| e.after)
                    .unwrap_or(ChannelExtrapolationMode::Hold),
                last_key,
                keys.len().checked_sub(2).and_then(|index| keys.get(index)),
                t,
            );
        }
        return last_key.value;
    }

    for key_index in 0..keys.len() - 1 {
        let left_key = &keys[key_index];
        let right_key = &keys[key_index + 1];
        if time == right_key.time {
            return right_key.value;
        }
        if time < left_key.time || time > right_key.time {
            continue;
        }

        if left_key.segment_to_next == ScalarSegmentType::Step {
            return left_key.value;
        }

        let span = (right_key.time - left_key.time).as_ticks() as f64;
        if span == 0.0 {
            return right_key.value;
        }

        let progress = ((t - left_key.time.as_ticks() as f64) / span).clamp(0.0, 1.0);
        if left_key.segment_to_next == ScalarSegmentType::Linear {
            return lerp_number(left_key.value, right_key.value, progress);
        }

        let curve_progress = solve_bezier_progress_for_time(time, left_key, right_key);
        let (right_handle, left_handle) = segment_handles(left_key, right_key);
        return bezier_point(
            curve_progress,
            left_key.value,
            left_key.value + right_handle.dv,
            right_key.value + left_handle.dv,
            right_key.value,
        );
    }

    last_key.value
}

pub fn discrete_channel_value_at_time(
    channel: Option<&DiscreteChannel>,
    time: MediaTime,
    fallback_value: DiscreteValue,
) -> DiscreteValue {
    let Some(channel) = channel.filter(|channel| !channel.keys.is_empty()) else {
        return fallback_value;
    };

    let normalized = normalize_discrete_channel(channel);
    let mut current_value = fallback_value;
    for key in &normalized.keys {
        if time < key.time {
            break;
        }
        current_value = key.value.clone();
    }
    current_value
}

pub fn channel_value_at_time(
    channel: Option<&AnimationChannel>,
    time: MediaTime,
    fallback_value: &ParamValue,
) -> ParamValue {
    let Some(channel) = channel.filter(|channel| match channel {
        AnimationChannel::Scalar(scalar) => !scalar.keys.is_empty(),
        AnimationChannel::Discrete(discrete) => !discrete.keys.is_empty(),
    }) else {
        return fallback_value.clone();
    };

    if let ParamValue::Number(number_fallback) = fallback_value {
        return match channel {
            AnimationChannel::Scalar(scalar) => ParamValue::Number(scalar_channel_value_at_time(
                Some(scalar),
                time,
                *number_fallback,
            )),
            AnimationChannel::Discrete(_) => fallback_value.clone(),
        };
    }

    match channel {
        AnimationChannel::Discrete(discrete) => {
            let discrete_fallback = match fallback_value {
                ParamValue::Bool(value) => DiscreteValue::Bool(*value),
                ParamValue::String(value) => DiscreteValue::String(value.clone()),
                ParamValue::Number(_) => return fallback_value.clone(),
            };
            discrete_channel_value_at_time(Some(discrete), time, discrete_fallback).into()
        }
        AnimationChannel::Scalar(_) => fallback_value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{ChannelExtrapolation, DiscreteAnimationKey, TangentMode};

    fn scalar_key(id: &str, time: i64, value: f64, segment: ScalarSegmentType) -> ScalarAnimationKey {
        ScalarAnimationKey {
            id: id.to_string(),
            time: MediaTime::from_ticks(time),
            value,
            left_handle: None,
            right_handle: None,
            segment_to_next: segment,
            tangent_mode: TangentMode::Flat,
        }
    }

    fn channel(keys: Vec<ScalarAnimationKey>) -> ScalarChannel {
        ScalarChannel {
            keys,
            extrapolation: None,
        }
    }

    fn at(time: i64) -> MediaTime {
        MediaTime::from_ticks(time)
    }

    #[test]
    fn empty_channel_returns_fallback() {
        assert_eq!(scalar_channel_value_at_time(None, at(10), 7.0), 7.0);
        assert_eq!(
            scalar_channel_value_at_time(Some(&channel(Vec::new())), at(10), 7.0),
            7.0
        );
    }

    #[test]
    fn linear_segment_interpolates() {
        let channel = channel(vec![
            scalar_key("a", 0, 0.0, ScalarSegmentType::Linear),
            scalar_key("b", 100, 10.0, ScalarSegmentType::Linear),
        ]);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(50), 0.0), 5.0);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(0), 0.0), 0.0);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(100), 0.0), 10.0);
    }

    #[test]
    fn step_segment_holds_left_value() {
        let channel = channel(vec![
            scalar_key("a", 0, 1.0, ScalarSegmentType::Step),
            scalar_key("b", 100, 9.0, ScalarSegmentType::Linear),
        ]);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(99), 0.0), 1.0);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(100), 0.0), 9.0);
    }

    #[test]
    fn unsorted_keys_are_sorted_before_sampling() {
        let channel = channel(vec![
            scalar_key("b", 100, 10.0, ScalarSegmentType::Linear),
            scalar_key("a", 0, 0.0, ScalarSegmentType::Linear),
        ]);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(50), 0.0), 5.0);
    }

    #[test]
    fn hold_extrapolation_is_the_default() {
        let channel = channel(vec![
            scalar_key("a", 10, 1.0, ScalarSegmentType::Linear),
            scalar_key("b", 100, 10.0, ScalarSegmentType::Linear),
        ]);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(0), 0.0), 1.0);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(200), 0.0), 10.0);
    }

    #[test]
    fn linear_extrapolation_extends_edge_segments() {
        let mut channel = channel(vec![
            scalar_key("a", 0, 0.0, ScalarSegmentType::Linear),
            scalar_key("b", 100, 10.0, ScalarSegmentType::Linear),
        ]);
        channel.extrapolation = Some(ChannelExtrapolation {
            before: ChannelExtrapolationMode::Linear,
            after: ChannelExtrapolationMode::Linear,
        });
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(-50), 0.0), -5.0);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(150), 0.0), 15.0);
    }

    #[test]
    fn linear_extrapolation_holds_with_single_key() {
        let mut channel = channel(vec![scalar_key("a", 10, 3.0, ScalarSegmentType::Linear)]);
        channel.extrapolation = Some(ChannelExtrapolation {
            before: ChannelExtrapolationMode::Linear,
            after: ChannelExtrapolationMode::Linear,
        });
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(0), 0.0), 3.0);
        assert_eq!(scalar_channel_value_at_time(Some(&channel), at(50), 0.0), 3.0);
    }

    #[test]
    fn bezier_segment_with_default_handles_matches_diagonal() {
        let channel = channel(vec![
            scalar_key("a", 0, 0.0, ScalarSegmentType::Bezier),
            scalar_key("b", 100, 100.0, ScalarSegmentType::Bezier),
        ]);
        let value = scalar_channel_value_at_time(Some(&channel), at(50), 0.0);
        assert!((value - 50.0).abs() < 1e-3);
    }

    #[test]
    fn bezier_segment_with_flat_handles_eases() {
        let mut left = scalar_key("a", 0, 0.0, ScalarSegmentType::Bezier);
        left.right_handle = Some(CurveHandle {
            dt: at(50),
            dv: 0.0,
        });
        let mut right = scalar_key("b", 100, 100.0, ScalarSegmentType::Bezier);
        right.left_handle = Some(CurveHandle {
            dt: at(-50),
            dv: 0.0,
        });
        let channel = channel(vec![left, right]);
        let quarter = scalar_channel_value_at_time(Some(&channel), at(25), 0.0);
        let mid = scalar_channel_value_at_time(Some(&channel), at(50), 0.0);
        assert!(quarter < 25.0);
        assert!((mid - 50.0).abs() < 1e-3);
    }

    #[test]
    fn handles_are_clamped_to_segment_span() {
        let mut left = scalar_key("a", 0, 0.0, ScalarSegmentType::Bezier);
        left.right_handle = Some(CurveHandle {
            dt: at(500),
            dv: 1.0,
        });
        let mut right = scalar_key("b", 100, 100.0, ScalarSegmentType::Bezier);
        right.left_handle = Some(CurveHandle {
            dt: at(50),
            dv: 1.0,
        });
        let normalized = normalize_scalar_channel(&channel(vec![left, right]));
        assert_eq!(normalized.keys[0].right_handle.unwrap().dt, at(100));
        assert_eq!(normalized.keys[1].left_handle.unwrap().dt, at(0));
    }

    #[test]
    fn edge_keys_drop_outer_handles() {
        let mut left = scalar_key("a", 0, 0.0, ScalarSegmentType::Bezier);
        left.left_handle = Some(CurveHandle {
            dt: at(-10),
            dv: 1.0,
        });
        let mut right = scalar_key("b", 100, 100.0, ScalarSegmentType::Bezier);
        right.right_handle = Some(CurveHandle {
            dt: at(10),
            dv: 1.0,
        });
        let normalized = normalize_scalar_channel(&channel(vec![left, right]));
        assert_eq!(normalized.keys[0].left_handle, None);
        assert_eq!(normalized.keys[1].right_handle, None);
    }

    #[test]
    fn discrete_channel_returns_last_key_at_or_before_time() {
        let channel = DiscreteChannel {
            keys: vec![
                DiscreteAnimationKey {
                    id: "a".to_string(),
                    time: at(0),
                    value: DiscreteValue::String("x".to_string()),
                },
                DiscreteAnimationKey {
                    id: "b".to_string(),
                    time: at(100),
                    value: DiscreteValue::Bool(true),
                },
            ],
        };
        let fallback = DiscreteValue::Bool(false);
        assert_eq!(
            discrete_channel_value_at_time(Some(&channel), at(-1), fallback.clone()),
            fallback
        );
        assert_eq!(
            discrete_channel_value_at_time(Some(&channel), at(50), fallback.clone()),
            DiscreteValue::String("x".to_string())
        );
        assert_eq!(
            discrete_channel_value_at_time(Some(&channel), at(100), fallback.clone()),
            DiscreteValue::Bool(true)
        );
    }

    #[test]
    fn channel_value_at_time_dispatches_on_fallback_type() {
        let scalar = AnimationChannel::Scalar(channel(vec![scalar_key(
            "a",
            0,
            2.0,
            ScalarSegmentType::Linear,
        )]));
        assert_eq!(
            channel_value_at_time(Some(&scalar), at(0), &ParamValue::Number(0.0)),
            ParamValue::Number(2.0)
        );
        assert_eq!(
            channel_value_at_time(Some(&scalar), at(0), &ParamValue::Bool(true)),
            ParamValue::Bool(true)
        );
    }
}
