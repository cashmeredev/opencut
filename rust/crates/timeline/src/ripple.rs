use std::collections::{HashMap, HashSet};

use scene::{Element, SceneTracks};
use time::MediaTime;

#[derive(Clone, Debug, PartialEq)]
pub struct RippleAdjustment {
    pub track_id: String,
    pub after_time: MediaTime,
    pub shift_amount: MediaTime,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Interval {
    start_time: MediaTime,
    end_time: MediaTime,
}

#[derive(Clone, Copy, Debug)]
struct ElementSpan {
    start_time: MediaTime,
    end_time: MediaTime,
}

pub fn compute_ripple_adjustments(
    before_tracks: &SceneTracks,
    after_tracks: &SceneTracks,
) -> Vec<RippleAdjustment> {
    let before_list = crate::placement::ordered_tracks(before_tracks);
    let after_list = crate::placement::ordered_tracks(after_tracks);
    let after_tracks_by_id: HashMap<&str, &scene::Track> = after_list
        .iter()
        .map(|track| (track.id(), *track))
        .collect();
    let all_after_element_ids: HashSet<&str> = after_list
        .iter()
        .flat_map(|track| track.elements().iter().map(|element| element.base().id.as_str()))
        .collect();

    let mut adjustments = Vec::new();
    for before_track in before_list {
        let after_elements = after_tracks_by_id
            .get(before_track.id())
            .map(|track| track.elements())
            .unwrap_or(&[]);
        adjustments.extend(compute_track_ripple_adjustments(
            before_track.id(),
            before_track.elements(),
            after_elements,
            &all_after_element_ids,
        ));
    }
    adjustments
}

fn compute_track_ripple_adjustments(
    track_id: &str,
    before_elements: &[Element],
    after_elements: &[Element],
    all_after_element_ids: &HashSet<&str>,
) -> Vec<RippleAdjustment> {
    let before_spans = build_element_span_map(before_elements);
    let after_spans = build_element_span_map(after_elements);
    let (vacated, joined) = collect_track_intervals(&before_spans, &after_spans, all_after_element_ids);
    let freed = subtract_interval_sets(&vacated, &joined);
    build_adjustments(track_id, &freed)
}

fn build_element_span_map(elements: &[Element]) -> HashMap<String, ElementSpan> {
    elements
        .iter()
        .map(|element| {
            (
                element.base().id.clone(),
                ElementSpan {
                    start_time: element.base().start_time,
                    end_time: element.base().start_time + element.base().duration,
                },
            )
        })
        .collect()
}

fn collect_track_intervals(
    before_spans: &HashMap<String, ElementSpan>,
    after_spans: &HashMap<String, ElementSpan>,
    all_after_element_ids: &HashSet<&str>,
) -> (Vec<Interval>, Vec<Interval>) {
    let mut vacated = Vec::new();
    let mut joined = Vec::new();

    for (id, before_span) in before_spans {
        match after_spans.get(id) {
            None => {
                if !all_after_element_ids.contains(id.as_str()) {
                    push_interval(&mut vacated, before_span.start_time, before_span.end_time);
                }
            }
            Some(after_span) => {
                if before_span.end_time > after_span.end_time {
                    push_interval(&mut vacated, after_span.end_time, before_span.end_time);
                }
            }
        }
    }

    for (id, after_span) in after_spans {
        if before_spans.contains_key(id) {
            continue;
        }
        push_interval(&mut joined, after_span.start_time, after_span.end_time);
    }

    (normalize_intervals(vacated), normalize_intervals(joined))
}

fn build_adjustments(track_id: &str, intervals: &[Interval]) -> Vec<RippleAdjustment> {
    intervals
        .iter()
        .filter_map(|interval| {
            let shift_amount = interval.end_time - interval.start_time;
            if shift_amount <= MediaTime::ZERO {
                return None;
            }
            Some(RippleAdjustment {
                track_id: track_id.to_string(),
                after_time: interval.end_time,
                shift_amount,
            })
        })
        .collect()
}

fn subtract_interval_sets(source: &[Interval], overlapping: &[Interval]) -> Vec<Interval> {
    let normalized_source = normalize_intervals(source.to_vec());
    let normalized_overlapping = normalize_intervals(overlapping.to_vec());

    normalized_source
        .iter()
        .flat_map(|source_interval| subtract_single_interval(*source_interval, &normalized_overlapping))
        .collect()
}

fn normalize_intervals(intervals: Vec<Interval>) -> Vec<Interval> {
    let mut valid = Vec::new();
    for interval in intervals {
        push_interval(&mut valid, interval.start_time, interval.end_time);
    }
    valid.sort_by_key(|interval| interval.start_time);

    let mut merged: Vec<Interval> = Vec::new();
    for interval in valid {
        match merged.last_mut() {
            Some(previous) if interval.start_time <= previous.end_time => {
                previous.end_time = previous.end_time.max(interval.end_time);
            }
            _ => merged.push(interval),
        }
    }
    merged
}

fn subtract_single_interval(source: Interval, overlapping: &[Interval]) -> Vec<Interval> {
    let mut remaining = vec![source];

    for overlapping_interval in overlapping {
        let mut next = Vec::new();
        for interval in remaining {
            if overlapping_interval.end_time <= interval.start_time
                || overlapping_interval.start_time >= interval.end_time
            {
                next.push(interval);
                continue;
            }

            push_interval(&mut next, interval.start_time, overlapping_interval.start_time);
            push_interval(&mut next, overlapping_interval.end_time, interval.end_time);
        }
        remaining = next;

        if remaining.is_empty() {
            return Vec::new();
        }
    }

    remaining
}

fn push_interval(intervals: &mut Vec<Interval>, start_time: MediaTime, end_time: MediaTime) {
    if end_time <= start_time {
        return;
    }
    intervals.push(Interval { start_time, end_time });
}

pub fn ripple_shift_elements(
    elements: &[Element],
    after_time: MediaTime,
    shift_amount: MediaTime,
) -> Vec<Element> {
    elements
        .iter()
        .map(|element| {
            if element.base().start_time >= after_time {
                let mut shifted = element.clone();
                shifted.base_mut().start_time = element.base().start_time - shift_amount;
                shifted
            } else {
                element.clone()
            }
        })
        .collect()
}

pub fn apply_ripple_adjustments(
    tracks: &SceneTracks,
    adjustments: &[RippleAdjustment],
) -> SceneTracks {
    if adjustments.is_empty() {
        return tracks.clone();
    }

    let mut adjustments_by_track: HashMap<&str, Vec<&RippleAdjustment>> = HashMap::new();
    for adjustment in adjustments {
        adjustments_by_track
            .entry(adjustment.track_id.as_str())
            .or_default()
            .push(adjustment);
    }

    let apply_to_track = |track: &scene::Track| {
        let track_adjustments = adjustments_by_track
            .get(track.id())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        apply_track_ripple_adjustments(track, track_adjustments)
    };

    SceneTracks {
        overlay: tracks.overlay.iter().map(apply_to_track).collect(),
        main: apply_to_track(&tracks.main),
        audio: tracks.audio.iter().map(apply_to_track).collect(),
    }
}

fn apply_track_ripple_adjustments(
    track: &scene::Track,
    adjustments: &[&RippleAdjustment],
) -> scene::Track {
    if adjustments.is_empty() {
        return track.clone();
    }

    let mut sorted: Vec<&RippleAdjustment> = adjustments.to_vec();
    sorted.sort_by_key(|adjustment| std::cmp::Reverse(adjustment.after_time));

    let mut elements = track.elements().to_vec();
    for adjustment in sorted {
        elements = ripple_shift_elements(&elements, adjustment.after_time, adjustment.shift_amount);
    }

    let mut next = track.clone();
    *next.elements_mut() = elements;
    next
}
