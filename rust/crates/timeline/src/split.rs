use scene::{Element, ElementRef, SceneTracks};
use time::MediaTime;

use crate::retime::{get_source_span_at_clip_time, round_media_time};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainSide {
    Both,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SplitElementsResult {
    pub tracks: SceneTracks,
    pub right_side_elements: Vec<ElementRef>,
}

pub fn split_elements(
    tracks: &SceneTracks,
    targets: &[ElementRef],
    split_time: MediaTime,
    retain_side: RetainSide,
) -> SplitElementsResult {
    let mut right_side_elements = Vec::new();

    let split_track = |track: &scene::Track, right_side_elements: &mut Vec<ElementRef>| {
        let has_targets = targets.iter().any(|target| target.track_id == track.id());
        if !has_targets {
            return track.clone();
        }

        let elements = track
            .elements()
            .iter()
            .flat_map(|element| {
                let should_split = targets
                    .iter()
                    .any(|target| target.element_id == element.base().id);
                if !should_split {
                    return vec![element.clone()];
                }
                split_element(track.id(), element, split_time, retain_side, right_side_elements)
            })
            .collect();

        let mut next = track.clone();
        *next.elements_mut() = elements;
        next
    };

    let updated = SceneTracks {
        overlay: tracks
            .overlay
            .iter()
            .map(|track| split_track(track, &mut right_side_elements))
            .collect(),
        main: split_track(&tracks.main, &mut right_side_elements),
        audio: tracks
            .audio
            .iter()
            .map(|track| split_track(track, &mut right_side_elements))
            .collect(),
    };

    SplitElementsResult {
        tracks: updated,
        right_side_elements,
    }
}

fn split_element(
    track_id: &str,
    element: &Element,
    split_time: MediaTime,
    retain_side: RetainSide,
    right_side_elements: &mut Vec<ElementRef>,
) -> Vec<Element> {
    let base = element.base();
    let effective_start = base.start_time;
    let effective_end = base.start_time + base.duration;

    if split_time <= effective_start || split_time >= effective_end {
        return vec![element.clone()];
    }

    let relative_time = split_time - base.start_time;
    let left_visible_duration = relative_time;
    let right_visible_duration = base.duration - relative_time;
    let retime = crate::element_retime(element);

    let left_source_span = round_media_time(get_source_span_at_clip_time(
        left_visible_duration.as_ticks() as f64,
        retime.as_ref(),
    ));
    let total_source_span = round_media_time(get_source_span_at_clip_time(
        base.duration.as_ticks() as f64,
        retime.as_ref(),
    ));
    let right_source_span = total_source_span - left_source_span;

    let (left_animations, right_animations) = animation::split_animations_at_time_with_options(
        base.animations.as_ref(),
        relative_time,
        true,
    );

    let left_trim_end = base.trim_end + right_source_span;
    let right_trim_start = base.trim_start + left_source_span;

    let build_left = || {
        let mut left = element.clone();
        let left_base = left.base_mut();
        left_base.duration = left_visible_duration;
        left_base.trim_end = left_trim_end;
        left_base.name = format!("{} (left)", left_base.name);
        left_base.animations = left_animations;
        left
    };

    let build_right = |right_side_elements: &mut Vec<ElementRef>| {
        let new_id = animation::generate_uuid();
        right_side_elements.push(ElementRef {
            track_id: track_id.to_string(),
            element_id: new_id.clone(),
        });
        let mut right = element.clone();
        let right_base = right.base_mut();
        right_base.id = new_id;
        right_base.start_time = split_time;
        right_base.duration = right_visible_duration;
        right_base.trim_start = right_trim_start;
        right_base.name = format!("{} (right)", right_base.name);
        right_base.animations = right_animations;
        right
    };

    match retain_side {
        RetainSide::Left => vec![build_left()],
        RetainSide::Right => vec![build_right(right_side_elements)],
        RetainSide::Both => vec![build_left(), build_right(right_side_elements)],
    }
}
