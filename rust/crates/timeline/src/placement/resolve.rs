use scene::{SceneTracks, Track};
use time::MediaTime;

use super::compatibility::get_track_type_for_element_type;
use super::insert_index::{
    get_default_insert_index_for_track, get_highest_insert_index_for_track,
    resolve_preferred_new_track_placement,
};
use super::main_track::enforce_main_track_start;
use super::overlap::can_place_time_spans_on_track;
use super::{
    InsertDirection, NewTrackPosition, PlacementResult, PlacementStrategy, PlacementSubject,
    PlacementTimeSpan, VerticalDragDirection, ordered_tracks,
};
use crate::types::TrackType;

pub struct ResolveTrackPlacementParams {
    pub tracks: SceneTracks,
    pub subject: PlacementSubject,
    pub time_spans: Vec<PlacementTimeSpan>,
    pub strategy: PlacementStrategy,
}

fn build_existing_track_result(
    track: &Track,
    track_index: usize,
    tracks: &SceneTracks,
    time_spans: &[PlacementTimeSpan],
) -> PlacementResult {
    let first_span = time_spans.first();
    let requested_start_time = first_span
        .map(|span| span.start_time)
        .unwrap_or(MediaTime::ZERO);
    let adjusted_start_time = enforce_main_track_start(
        tracks,
        track.id(),
        requested_start_time,
        first_span.and_then(|span| span.exclude_element_id.as_deref()),
    );

    PlacementResult::ExistingTrack {
        track_id: track.id().to_string(),
        track_index,
        track_type: TrackType::of_track(track),
        adjusted_start_time: (adjusted_start_time != requested_start_time)
            .then_some(adjusted_start_time),
    }
}

fn build_new_track_result(
    track_type: TrackType,
    insert_index: usize,
    insert_position: Option<InsertDirection>,
) -> PlacementResult {
    PlacementResult::NewTrack {
        track_type,
        insert_index,
        insert_position,
    }
}

fn find_first_available_track_index(
    tracks: &[&Track],
    track_type: TrackType,
    time_spans: &[PlacementTimeSpan],
) -> Option<usize> {
    tracks.iter().position(|track| {
        TrackType::of_track(track) == track_type
            && can_place_time_spans_on_track(track.elements(), time_spans)
    })
}

fn resolve_always_new_track(
    tracks: &SceneTracks,
    track_type: TrackType,
    position: NewTrackPosition,
) -> PlacementResult {
    let insert_index = match position {
        NewTrackPosition::Highest => get_highest_insert_index_for_track(tracks, track_type),
        NewTrackPosition::Default => get_default_insert_index_for_track(tracks, track_type),
    };

    build_new_track_result(track_type, insert_index, None)
}

fn get_insert_direction(
    hover_direction: InsertDirection,
    vertical_drag_direction: Option<VerticalDragDirection>,
) -> InsertDirection {
    match vertical_drag_direction {
        Some(VerticalDragDirection::Up) => InsertDirection::Above,
        Some(VerticalDragDirection::Down) => InsertDirection::Below,
        None => hover_direction,
    }
}

pub fn resolve_track_placement(params: &ResolveTrackPlacementParams) -> Option<PlacementResult> {
    let tracks = &params.tracks;
    let ordered = ordered_tracks(tracks);
    let track_type = match params.subject {
        PlacementSubject::TrackType(track_type) => track_type,
        PlacementSubject::ElementType(element_type) => {
            get_track_type_for_element_type(element_type)
        }
    };
    let time_spans = &params.time_spans;

    match &params.strategy {
        PlacementStrategy::Explicit { track_id } => {
            let track_index = ordered.iter().position(|track| track.id() == track_id)?;
            let track = ordered[track_index];
            if TrackType::of_track(track) != track_type {
                return None;
            }

            Some(build_existing_track_result(
                track,
                track_index,
                tracks,
                time_spans,
            ))
        }
        PlacementStrategy::FirstAvailable => {
            if let Some(existing_index) =
                find_first_available_track_index(&ordered, track_type, time_spans)
            {
                return Some(build_existing_track_result(
                    ordered[existing_index],
                    existing_index,
                    tracks,
                    time_spans,
                ));
            }

            Some(resolve_always_new_track(
                tracks,
                track_type,
                NewTrackPosition::Highest,
            ))
        }
        PlacementStrategy::PreferIndex {
            track_index,
            hover_direction,
            vertical_drag_direction,
            create_new_track_only,
        } => {
            let preferred_track = ordered.get(*track_index);
            let is_preferred_compatible = preferred_track
                .is_some_and(|track| TrackType::of_track(track) == track_type);
            let can_use_existing = !create_new_track_only
                && is_preferred_compatible
                && preferred_track.is_some_and(|track| {
                    can_place_time_spans_on_track(track.elements(), time_spans)
                });
            if can_use_existing {
                return Some(build_existing_track_result(
                    preferred_track?,
                    *track_index,
                    tracks,
                    time_spans,
                ));
            }

            let direction = get_insert_direction(
                *hover_direction,
                if is_preferred_compatible {
                    None
                } else {
                    *vertical_drag_direction
                },
            );
            let placement = resolve_preferred_new_track_placement(
                tracks,
                track_type,
                *track_index,
                direction,
            );
            Some(build_new_track_result(
                track_type,
                placement.insert_index,
                placement.insert_position,
            ))
        }
        PlacementStrategy::AboveSource { source_track_index } => {
            let above_track = source_track_index
                .checked_sub(1)
                .and_then(|above_index| ordered.get(above_index));
            if let Some(above_track) = above_track
                && TrackType::of_track(above_track) == track_type
                && can_place_time_spans_on_track(above_track.elements(), time_spans)
            {
                return Some(build_existing_track_result(
                    above_track,
                    source_track_index - 1,
                    tracks,
                    time_spans,
                ));
            }

            if let Some(first_available_index) =
                find_first_available_track_index(&ordered, track_type, time_spans)
            {
                return Some(build_existing_track_result(
                    ordered[first_available_index],
                    first_available_index,
                    tracks,
                    time_spans,
                ));
            }

            Some(build_new_track_result(
                track_type,
                get_highest_insert_index_for_track(tracks, track_type),
                None,
            ))
        }
        PlacementStrategy::AlwaysNew { position } => Some(resolve_always_new_track(
            tracks,
            track_type,
            *position,
        )),
    }
}
