use scene::{ElementRef, RetimeConfig, SceneTracks};
use time::{FrameRate, MediaTime, TICKS_PER_SECOND};

use crate::retime::{
    get_source_span_at_clip_time, get_timeline_duration_for_source_span, round_media_time,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizeSide {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupResizePushTarget {
    pub track_id: String,
    pub element_id: String,
    pub start_time: MediaTime,
    pub duration: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupResizeMember {
    pub track_id: String,
    pub element_id: String,
    pub start_time: MediaTime,
    pub duration: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    pub source_duration: Option<MediaTime>,
    pub retime: Option<RetimeConfig>,
    pub left_neighbor_bound: Option<MediaTime>,
    pub right_neighbor_bound: Option<MediaTime>,
    pub right_push_chain: Option<Vec<GroupResizePushTarget>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupResizePatch {
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    pub start_time: MediaTime,
    pub duration: MediaTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupResizeUpdate {
    pub track_id: String,
    pub element_id: String,
    pub patch: GroupResizePatch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupResizeResult {
    pub delta_time: MediaTime,
    pub updates: Vec<GroupResizeUpdate>,
}

pub fn compute_group_resize(
    members: &[GroupResizeMember],
    side: ResizeSide,
    delta_time: MediaTime,
    fps: FrameRate,
) -> GroupResizeResult {
    if members.is_empty() {
        return GroupResizeResult {
            delta_time: MediaTime::ZERO,
            updates: Vec::new(),
        };
    }

    let min_duration = MediaTime::from_ticks(
        ((TICKS_PER_SECOND as f64 * fps.denominator as f64) / fps.numerator as f64).round() as i64,
    );
    let mut minimum_delta_time = get_minimum_allowed_delta_time(&members[0], side, min_duration);
    let mut maximum_delta_time = get_maximum_allowed_delta_time(&members[0], side, min_duration);

    for member in &members[1..] {
        minimum_delta_time =
            minimum_delta_time.max(get_minimum_allowed_delta_time(member, side, min_duration));
        if let Some(member_maximum) = get_maximum_allowed_delta_time(member, side, min_duration) {
            maximum_delta_time = Some(match maximum_delta_time {
                None => member_maximum,
                Some(current) => current.min(member_maximum),
            });
        }
    }

    let clamped_delta_time = match maximum_delta_time {
        None => minimum_delta_time.max(delta_time),
        Some(maximum) => delta_time.clamp(minimum_delta_time, maximum),
    };

    let snapped_delta_time = clamped_delta_time
        .round_to_frame(fps)
        .unwrap_or(clamped_delta_time);
    let final_delta_time = match maximum_delta_time {
        None => minimum_delta_time.max(snapped_delta_time),
        Some(maximum) => snapped_delta_time.clamp(minimum_delta_time, maximum),
    };

    let mut updates: Vec<GroupResizeUpdate> = members
        .iter()
        .map(|member| build_resize_update(member, side, final_delta_time))
        .collect();
    updates.extend(build_ripple_push_updates(members, side, final_delta_time));

    GroupResizeResult {
        delta_time: final_delta_time,
        updates,
    }
}

fn build_ripple_push_updates(
    members: &[GroupResizeMember],
    side: ResizeSide,
    delta_time: MediaTime,
) -> Vec<GroupResizeUpdate> {
    if side != ResizeSide::Right || delta_time <= MediaTime::ZERO {
        return Vec::new();
    }

    let mut overflow_by_target: Vec<(GroupResizePushTarget, MediaTime)> = Vec::new();
    for member in members {
        let (Some(right_neighbor_bound), Some(push_chain)) =
            (member.right_neighbor_bound, &member.right_push_chain)
        else {
            continue;
        };
        if member.source_duration.is_some() {
            continue;
        }

        let neighbor_ceiling = right_neighbor_bound - (member.start_time + member.duration);
        let overflow = delta_time - neighbor_ceiling;
        if overflow <= MediaTime::ZERO {
            continue;
        }
        for target in push_chain {
            match overflow_by_target
                .iter_mut()
                .find(|(existing, _)| existing.element_id == target.element_id)
            {
                Some((_, existing_overflow)) => {
                    if overflow > *existing_overflow {
                        *existing_overflow = overflow;
                    }
                }
                None => overflow_by_target.push((target.clone(), overflow)),
            }
        }
    }

    overflow_by_target
        .into_iter()
        .map(|(target, overflow)| GroupResizeUpdate {
            track_id: target.track_id,
            element_id: target.element_id,
            patch: GroupResizePatch {
                trim_start: target.trim_start,
                trim_end: target.trim_end,
                start_time: target.start_time + overflow,
                duration: target.duration,
            },
        })
        .collect()
}

fn build_resize_update(
    member: &GroupResizeMember,
    side: ResizeSide,
    delta_time: MediaTime,
) -> GroupResizeUpdate {
    let source_delta = get_source_delta_for_clip_delta(member, delta_time);

    match side {
        ResizeSide::Left => GroupResizeUpdate {
            track_id: member.track_id.clone(),
            element_id: member.element_id.clone(),
            patch: GroupResizePatch {
                trim_start: MediaTime::ZERO.max(member.trim_start + source_delta),
                trim_end: member.trim_end,
                start_time: member.start_time + delta_time,
                duration: member.duration - delta_time,
            },
        },
        ResizeSide::Right => GroupResizeUpdate {
            track_id: member.track_id.clone(),
            element_id: member.element_id.clone(),
            patch: GroupResizePatch {
                trim_start: member.trim_start,
                trim_end: MediaTime::ZERO.max(member.trim_end - source_delta),
                start_time: member.start_time,
                duration: member.duration + delta_time,
            },
        },
    }
}

fn get_minimum_allowed_delta_time(
    member: &GroupResizeMember,
    side: ResizeSide,
    min_duration: MediaTime,
) -> MediaTime {
    if side == ResizeSide::Right {
        return min_duration - member.duration;
    }

    let left_neighbor_floor = match member.left_neighbor_bound {
        Some(bound) => bound - member.start_time,
        None => MediaTime::ZERO - member.start_time,
    };
    if member.source_duration.is_none() {
        return left_neighbor_floor;
    }

    let maximum_source_extension = get_duration_for_visible_source_span(
        member,
        get_visible_source_span_for_duration(member, member.duration) + member.trim_start,
    ) - member.duration;
    left_neighbor_floor.max(MediaTime::ZERO - maximum_source_extension)
}

fn get_maximum_allowed_delta_time(
    member: &GroupResizeMember,
    side: ResizeSide,
    min_duration: MediaTime,
) -> Option<MediaTime> {
    if side == ResizeSide::Left {
        return Some(member.duration - min_duration);
    }

    let right_neighbor_ceiling = member
        .right_neighbor_bound
        .map(|bound| bound - (member.start_time + member.duration));
    if member.source_duration.is_none() {
        return if member.right_push_chain.is_none() {
            right_neighbor_ceiling
        } else {
            None
        };
    }

    let maximum_visible_source_span = get_source_duration(member) - member.trim_start;
    let maximum_duration = get_duration_for_visible_source_span(member, maximum_visible_source_span);
    let source_duration_ceiling = maximum_duration - member.duration;
    Some(match right_neighbor_ceiling {
        None => source_duration_ceiling,
        Some(ceiling) => ceiling.min(source_duration_ceiling),
    })
}

fn get_source_delta_for_clip_delta(member: &GroupResizeMember, clip_delta: MediaTime) -> MediaTime {
    let Some(retime) = &member.retime else {
        return clip_delta;
    };

    let source_delta = if clip_delta >= MediaTime::ZERO {
        get_source_span_at_clip_time(clip_delta.as_ticks() as f64, Some(retime))
    } else {
        -get_source_span_at_clip_time(clip_delta.as_ticks().unsigned_abs() as f64, Some(retime))
    };
    round_media_time(source_delta)
}

fn get_visible_source_span_for_duration(member: &GroupResizeMember, duration: MediaTime) -> MediaTime {
    let Some(retime) = &member.retime else {
        return duration;
    };

    round_media_time(get_source_span_at_clip_time(
        duration.as_ticks() as f64,
        Some(retime),
    ))
}

fn get_duration_for_visible_source_span(
    member: &GroupResizeMember,
    source_span: MediaTime,
) -> MediaTime {
    let Some(retime) = &member.retime else {
        return source_span;
    };

    round_media_time(get_timeline_duration_for_source_span(
        source_span.as_ticks() as f64,
        Some(retime),
    ))
}

fn get_source_duration(member: &GroupResizeMember) -> MediaTime {
    if let Some(source_duration) = member.source_duration {
        return source_duration;
    }

    member.trim_start
        + get_visible_source_span_for_duration(member, member.duration)
        + member.trim_end
}

pub fn build_resize_members(
    tracks: &SceneTracks,
    selected_elements: &[ElementRef],
) -> Vec<GroupResizeMember> {
    let selected_ids: Vec<&str> = selected_elements
        .iter()
        .map(|reference| reference.element_id.as_str())
        .collect();

    let mut members = Vec::new();
    for reference in selected_elements {
        let Some(track) = crate::track_element_update::find_track_in_scene_tracks(
            tracks,
            &reference.track_id,
        ) else {
            continue;
        };
        let Some(element) = track
            .elements()
            .iter()
            .find(|element| element.base().id == reference.element_id)
        else {
            continue;
        };

        let base = element.base();
        let other_elements: Vec<&scene::Element> = track
            .elements()
            .iter()
            .filter(|candidate| !selected_ids.contains(&candidate.base().id.as_str()))
            .collect();

        let left_neighbor_bound = other_elements
            .iter()
            .filter(|candidate| {
                candidate.base().start_time + candidate.base().duration <= base.start_time
            })
            .map(|candidate| candidate.base().start_time + candidate.base().duration)
            .reduce(MediaTime::max);

        let mut right_elements: Vec<&scene::Element> = other_elements
            .iter()
            .filter(|candidate| candidate.base().start_time >= base.start_time + base.duration)
            .copied()
            .collect();
        let right_neighbor_bound = right_elements
            .iter()
            .map(|candidate| candidate.base().start_time)
            .reduce(MediaTime::min);

        let right_push_chain = if base.source_duration.is_none() {
            right_elements.sort_by_key(|candidate| candidate.base().start_time);
            Some(
                right_elements
                    .iter()
                    .map(|candidate| GroupResizePushTarget {
                        track_id: reference.track_id.clone(),
                        element_id: candidate.base().id.clone(),
                        start_time: candidate.base().start_time,
                        duration: candidate.base().duration,
                        trim_start: candidate.base().trim_start,
                        trim_end: candidate.base().trim_end,
                    })
                    .collect(),
            )
        } else {
            None
        };

        members.push(GroupResizeMember {
            track_id: reference.track_id.clone(),
            element_id: reference.element_id.clone(),
            start_time: base.start_time,
            duration: base.duration,
            trim_start: base.trim_start,
            trim_end: base.trim_end,
            source_duration: base.source_duration,
            retime: crate::element_retime(element),
            left_neighbor_bound,
            right_neighbor_bound,
            right_push_chain,
        });
    }

    members
}

pub fn has_resize_changes(members: &[GroupResizeMember], result: &GroupResizeResult) -> bool {
    result.updates.iter().any(|update| {
        let member = members
            .iter()
            .find(|member| member.element_id == update.element_id);
        member.is_some_and(|member| {
            member.trim_start != update.patch.trim_start
                || member.trim_end != update.patch.trim_end
                || member.start_time != update.patch.start_time
                || member.duration != update.patch.duration
        })
    })
}
