use scene::{AnimationChannel, ChannelData, ElementAnimations, ScalarAnimationKey, ScalarChannel};
use serde::{Deserialize, Serialize};

use crate::channel_data::get_channel_entries_from_data;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChannelEasingMode {
    Independent,
    Shared,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScalarGraphChannel {
    pub property_path: String,
    pub component_key: String,
    pub channel: ScalarChannel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditableScalarChannels {
    pub easing_mode: ChannelEasingMode,
    pub channels: Vec<ScalarGraphChannel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScalarGraphKeyframeContext {
    pub property_path: String,
    pub component_key: String,
    pub channel: ScalarChannel,
    pub keyframe: ScalarAnimationKey,
    pub keyframe_index: usize,
    pub previous_key: Option<ScalarAnimationKey>,
    pub next_key: Option<ScalarAnimationKey>,
}

fn easing_mode_for_channel_data(data: &ChannelData) -> ChannelEasingMode {
    match data {
        ChannelData::Composite(components)
            if ["r", "g", "b", "a"]
                .iter()
                .all(|key| components.contains_key(*key)) =>
        {
            ChannelEasingMode::Shared
        }
        _ => ChannelEasingMode::Independent,
    }
}

pub fn get_editable_scalar_channels(
    animations: Option<&ElementAnimations>,
    property_path: &str,
) -> Option<EditableScalarChannels> {
    let data = animations?.get(property_path)?;

    let channels = get_channel_entries_from_data(Some(data))
        .into_iter()
        .filter_map(|(component_key, channel)| match channel {
            AnimationChannel::Scalar(scalar) => Some(ScalarGraphChannel {
                property_path: property_path.to_string(),
                component_key,
                channel: scalar.clone(),
            }),
            AnimationChannel::Discrete(_) => None,
        })
        .collect();

    Some(EditableScalarChannels {
        easing_mode: easing_mode_for_channel_data(data),
        channels,
    })
}

pub fn get_editable_scalar_channel(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    component_key: &str,
) -> Option<ScalarGraphChannel> {
    get_editable_scalar_channels(animations, property_path)?
        .channels
        .into_iter()
        .find(|channel| channel.component_key == component_key)
}

pub fn get_scalar_keyframe_context(
    animations: Option<&ElementAnimations>,
    property_path: &str,
    component_key: &str,
    keyframe_id: &str,
) -> Option<ScalarGraphKeyframeContext> {
    let graph_channel = get_editable_scalar_channel(animations, property_path, component_key)?;
    let keyframe_index = graph_channel
        .channel
        .keys
        .iter()
        .position(|key| key.id == keyframe_id)?;

    Some(ScalarGraphKeyframeContext {
        property_path: graph_channel.property_path,
        component_key: graph_channel.component_key,
        keyframe: graph_channel.channel.keys[keyframe_index].clone(),
        keyframe_index,
        previous_key: keyframe_index
            .checked_sub(1)
            .and_then(|index| graph_channel.channel.keys.get(index))
            .cloned(),
        next_key: graph_channel.channel.keys.get(keyframe_index + 1).cloned(),
        channel: graph_channel.channel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{ScalarSegmentType, TangentMode};
    use std::collections::BTreeMap;
    use time::MediaTime;

    fn key(id: &str, time: i64) -> ScalarAnimationKey {
        ScalarAnimationKey {
            id: id.to_string(),
            time: MediaTime::from_ticks(time),
            value: 0.0,
            left_handle: None,
            right_handle: None,
            segment_to_next: ScalarSegmentType::Linear,
            tangent_mode: TangentMode::Flat,
        }
    }

    fn scalar_channel(keys: Vec<ScalarAnimationKey>) -> AnimationChannel {
        AnimationChannel::Scalar(ScalarChannel {
            keys,
            extrapolation: None,
        })
    }

    #[test]
    fn exposes_scalar_channels_for_editing() {
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert(
            "opacity".to_string(),
            ChannelData::Single(scalar_channel(vec![key("a", 0), key("b", 10)])),
        );
        let editable = get_editable_scalar_channels(Some(&animations), "opacity").unwrap();
        assert_eq!(editable.easing_mode, ChannelEasingMode::Independent);
        assert_eq!(editable.channels.len(), 1);
        assert_eq!(editable.channels[0].component_key, "value");

        let context =
            get_scalar_keyframe_context(Some(&animations), "opacity", "value", "b").unwrap();
        assert_eq!(context.keyframe_index, 1);
        assert_eq!(context.previous_key.as_ref().map(|key| key.id.as_str()), Some("a"));
        assert_eq!(context.next_key, None);
    }

    #[test]
    fn color_composites_share_easing() {
        let mut components = BTreeMap::new();
        for component_key in ["r", "g", "b", "a"] {
            components.insert(component_key.to_string(), scalar_channel(vec![key("k", 0)]));
        }
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert("color".to_string(), ChannelData::Composite(components));
        let editable = get_editable_scalar_channels(Some(&animations), "color").unwrap();
        assert_eq!(editable.easing_mode, ChannelEasingMode::Shared);
        assert_eq!(editable.channels.len(), 4);
    }

    #[test]
    fn discrete_channels_are_not_editable_scalars() {
        let channel = AnimationChannel::Discrete(scene::DiscreteChannel {
            keys: vec![scene::DiscreteAnimationKey {
                id: "d".to_string(),
                time: MediaTime::from_ticks(0),
                value: scene::DiscreteValue::Bool(true),
            }],
        });
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert("params.visible".to_string(), ChannelData::Single(channel));
        let editable = get_editable_scalar_channels(Some(&animations), "params.visible").unwrap();
        assert!(editable.channels.is_empty());
    }
}
