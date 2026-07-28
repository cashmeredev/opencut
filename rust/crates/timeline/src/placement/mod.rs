use scene::{SceneTracks, Track};
use time::MediaTime;

use crate::types::{ElementType, TrackType};

pub mod apply;
pub mod compatibility;
pub mod insert_index;
pub mod main_track;
pub mod overlap;
pub mod resolve;
pub mod track_factory;

pub use apply::{PlacementApplied, apply_placement};
pub use compatibility::{
    can_element_go_on_track, get_track_type_for_element_type, validate_element_track_compatibility,
};
pub use insert_index::{
    get_default_insert_index_for_track, get_highest_insert_index_for_track,
    resolve_preferred_new_track_placement,
};
pub use main_track::{MAIN_TRACK_NAME, enforce_main_track_start, get_earliest_main_track_element};
pub use overlap::can_place_time_spans_on_track;
pub use resolve::{ResolveTrackPlacementParams, resolve_track_placement};
pub use track_factory::build_empty_track;

#[derive(Clone, Debug, PartialEq)]
pub struct PlacementTimeSpan {
    pub start_time: MediaTime,
    pub duration: MediaTime,
    pub exclude_element_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlacementSubject {
    ElementType(ElementType),
    TrackType(TrackType),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsertDirection {
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalDragDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NewTrackPosition {
    Highest,
    Default,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlacementStrategy {
    Explicit {
        track_id: String,
    },
    FirstAvailable,
    PreferIndex {
        track_index: usize,
        hover_direction: InsertDirection,
        vertical_drag_direction: Option<VerticalDragDirection>,
        create_new_track_only: bool,
    },
    AboveSource {
        source_track_index: usize,
    },
    AlwaysNew {
        position: NewTrackPosition,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlacementResult {
    ExistingTrack {
        track_id: String,
        track_index: usize,
        track_type: TrackType,
        adjusted_start_time: Option<MediaTime>,
    },
    NewTrack {
        track_type: TrackType,
        insert_index: usize,
        insert_position: Option<InsertDirection>,
    },
}

pub fn ordered_tracks(tracks: &SceneTracks) -> Vec<&Track> {
    tracks
        .overlay
        .iter()
        .chain(std::iter::once(&tracks.main))
        .chain(tracks.audio.iter())
        .collect()
}
