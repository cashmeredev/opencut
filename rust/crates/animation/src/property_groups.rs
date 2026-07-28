use scene::{ANIMATION_PROPERTY_PATHS, ElementAnimations};
use time::MediaTime;

use crate::keyframe_query::get_keyframe_at_time;

pub const ANIMATION_PROPERTY_GROUPS: &[(&str, &[&str])] = &[(
    "transform.scale",
    &["transform.scaleX", "transform.scaleY"],
)];

#[derive(Clone, Debug, PartialEq)]
pub struct GroupKeyframeRef {
    pub property_path: &'static str,
    pub keyframe_id: String,
}

pub fn animation_property_group_paths(group: &str) -> Option<&'static [&'static str]> {
    ANIMATION_PROPERTY_GROUPS
        .iter()
        .find(|(name, _)| *name == group)
        .map(|(_, paths)| *paths)
}

pub fn get_group_keyframes_at_time(
    animations: Option<&ElementAnimations>,
    group: &str,
    time: MediaTime,
) -> Vec<GroupKeyframeRef> {
    let Some(paths) = animation_property_group_paths(group) else {
        return Vec::new();
    };
    paths
        .iter()
        .filter_map(|property_path| {
            get_keyframe_at_time(animations, property_path, time).map(|keyframe| {
                GroupKeyframeRef {
                    property_path,
                    keyframe_id: keyframe.id,
                }
            })
        })
        .collect()
}

pub fn has_group_keyframe_at_time(
    animations: Option<&ElementAnimations>,
    group: &str,
    time: MediaTime,
) -> bool {
    !get_group_keyframes_at_time(animations, group, time).is_empty()
}

pub fn is_animation_property_path(property_path: &str) -> bool {
    ANIMATION_PROPERTY_PATHS.contains(&property_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{
        AnimationChannel, ChannelData, ScalarAnimationKey, ScalarChannel, ScalarSegmentType,
        TangentMode,
    };
    use std::collections::BTreeMap;

    #[test]
    fn finds_group_keyframes_across_group_paths() {
        let key = |id: &str| ScalarAnimationKey {
            id: id.to_string(),
            time: MediaTime::from_ticks(0),
            value: 1.0,
            left_handle: None,
            right_handle: None,
            segment_to_next: ScalarSegmentType::Linear,
            tangent_mode: TangentMode::Flat,
        };
        let mut animations: ElementAnimations = BTreeMap::new();
        animations.insert(
            "transform.scaleX".to_string(),
            ChannelData::Single(AnimationChannel::Scalar(ScalarChannel {
                keys: vec![key("kx")],
                extrapolation: None,
            })),
        );
        let refs = get_group_keyframes_at_time(
            Some(&animations),
            "transform.scale",
            MediaTime::from_ticks(0),
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].property_path, "transform.scaleX");
        assert!(has_group_keyframe_at_time(
            Some(&animations),
            "transform.scale",
            MediaTime::from_ticks(0)
        ));
        assert!(!has_group_keyframe_at_time(
            Some(&animations),
            "transform.scale",
            MediaTime::from_ticks(10)
        ));
    }

    #[test]
    fn recognizes_animation_property_paths() {
        assert!(is_animation_property_path("opacity"));
        assert!(is_animation_property_path("transform.rotate"));
        assert!(!is_animation_property_path("params.blur"));
    }
}
