use std::collections::HashSet;

use scene::{Element, ElementAnimations, ParamValues, RetimeConfig, SceneTracks};
use time::MediaTime;

use crate::retime::{
    clamp_retime_rate, get_source_span_at_clip_time, get_timeline_duration_for_source_span,
    round_media_time,
};

#[derive(Clone, Debug, Default)]
pub struct ElementPatch {
    pub name: Option<String>,
    pub duration: Option<MediaTime>,
    pub start_time: Option<MediaTime>,
    pub trim_start: Option<MediaTime>,
    pub trim_end: Option<MediaTime>,
    pub source_duration: Option<Option<MediaTime>>,
    pub animations: Option<Option<ElementAnimations>>,
    pub params: Option<ParamValues>,
    pub retime: Option<Option<RetimeConfig>>,
}

pub struct ElementUpdateContext<'a> {
    pub tracks: &'a SceneTracks,
    pub track_id: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum UpdateField {
    Retime,
    Duration,
    StartTime,
}

pub fn apply_element_update(
    element: &Element,
    patch: &ElementPatch,
    context: &ElementUpdateContext,
) -> Element {
    let mut next_element = merge_patch(element, patch);
    let mut changed_fields: HashSet<UpdateField> = HashSet::new();
    if patch.retime.is_some() {
        changed_fields.insert(UpdateField::Retime);
    }
    if patch.duration.is_some() {
        changed_fields.insert(UpdateField::Duration);
    }
    if patch.start_time.is_some() {
        changed_fields.insert(UpdateField::StartTime);
    }

    if changed_fields.contains(&UpdateField::Retime)
        && let Some(patch_retime) = &patch.retime
        && crate::is_retimable_element(&next_element)
    {
        let next_retime = patch_retime.map(|retime| RetimeConfig {
            rate: clamp_retime_rate(retime.rate),
            maintain_pitch: retime.maintain_pitch,
        });

        let source_duration = get_source_duration(element);
        let visible_source_span = (source_duration
            - next_element.base().trim_start.as_ticks() as f64
            - next_element.base().trim_end.as_ticks() as f64)
            .max(0.0);
        let next_duration = round_media_time(get_timeline_duration_for_source_span(
            visible_source_span,
            next_retime.as_ref(),
        ));

        crate::set_element_retime(&mut next_element, next_retime);
        next_element.base_mut().duration = next_duration;
        changed_fields.insert(UpdateField::Duration);
    }

    if changed_fields.contains(&UpdateField::Duration) {
        next_element.base_mut().animations = animation::clamp_animations_to_duration(
            next_element.base().animations.as_ref(),
            next_element.base().duration,
        );
    }

    if changed_fields.contains(&UpdateField::StartTime) {
        let requested_start_time = next_element.base().start_time.max(MediaTime::ZERO);
        if context.track_id != context.tracks.main.id() {
            next_element.base_mut().start_time = requested_start_time;
        } else {
            let earliest = context
                .tracks
                .main
                .elements()
                .iter()
                .filter(|candidate| candidate.base().id != next_element.base().id)
                .reduce(|earliest, candidate| {
                    if candidate.base().start_time < earliest.base().start_time {
                        candidate
                    } else {
                        earliest
                    }
                });

            next_element.base_mut().start_time = match earliest {
                Some(earliest) if requested_start_time > earliest.base().start_time => {
                    requested_start_time
                }
                _ => MediaTime::ZERO,
            };
        }
    }

    next_element
}

fn merge_patch(element: &Element, patch: &ElementPatch) -> Element {
    let mut next = element.clone();
    let base = next.base_mut();
    if let Some(name) = &patch.name {
        base.name = name.clone();
    }
    if let Some(duration) = patch.duration {
        base.duration = duration;
    }
    if let Some(start_time) = patch.start_time {
        base.start_time = start_time;
    }
    if let Some(trim_start) = patch.trim_start {
        base.trim_start = trim_start;
    }
    if let Some(trim_end) = patch.trim_end {
        base.trim_end = trim_end;
    }
    if let Some(source_duration) = patch.source_duration {
        base.source_duration = source_duration;
    }
    if let Some(animations) = &patch.animations {
        base.animations = animations.clone();
    }
    if let Some(params) = &patch.params {
        base.params.extend(params.clone());
    }
    if let Some(retime) = &patch.retime {
        crate::set_element_retime(&mut next, *retime);
    }
    next
}

fn get_source_duration(element: &Element) -> f64 {
    let base = element.base();
    if let Some(source_duration) = base.source_duration {
        return source_duration.as_ticks() as f64;
    }

    let retime = crate::element_retime(element);
    base.trim_start.as_ticks() as f64
        + get_source_span_at_clip_time(base.duration.as_ticks() as f64, retime.as_ref())
        + base.trim_end.as_ticks() as f64
}
