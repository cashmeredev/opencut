use scene::SceneTracks;

use super::InsertDirection;
use crate::types::TrackType;

pub fn get_default_insert_index_for_track(tracks: &SceneTracks, track_type: TrackType) -> usize {
    if track_type == TrackType::Audio {
        return tracks.overlay.len() + 1 + tracks.audio.len();
    }

    if track_type == TrackType::Effect {
        return 0;
    }

    tracks.overlay.len()
}

pub fn get_highest_insert_index_for_track(tracks: &SceneTracks, track_type: TrackType) -> usize {
    if track_type == TrackType::Audio {
        return tracks.overlay.len() + 1;
    }

    0
}

pub struct PreferredNewTrackPlacement {
    pub insert_index: usize,
    pub insert_position: Option<InsertDirection>,
}

pub fn resolve_preferred_new_track_placement(
    tracks: &SceneTracks,
    track_type: TrackType,
    preferred_index: usize,
    direction: InsertDirection,
) -> PreferredNewTrackPlacement {
    let track_count = tracks.overlay.len() + 1 + tracks.audio.len();
    if track_count == 0 {
        return PreferredNewTrackPlacement {
            insert_index: 0,
            insert_position: (track_type == TrackType::Audio)
                .then_some(InsertDirection::Below),
        };
    }

    let safe_preferred_index = preferred_index.min(track_count - 1);
    let main_track_index = tracks.overlay.len();

    if track_type == TrackType::Audio {
        if safe_preferred_index <= main_track_index {
            return PreferredNewTrackPlacement {
                insert_index: main_track_index + 1,
                insert_position: Some(InsertDirection::Below),
            };
        }

        return PreferredNewTrackPlacement {
            insert_index: match direction {
                InsertDirection::Above => safe_preferred_index,
                InsertDirection::Below => safe_preferred_index + 1,
            },
            insert_position: Some(direction),
        };
    }

    let insert_index = match direction {
        InsertDirection::Above => safe_preferred_index,
        InsertDirection::Below => safe_preferred_index + 1,
    };
    if insert_index > main_track_index {
        return PreferredNewTrackPlacement {
            insert_index: main_track_index,
            insert_position: Some(InsertDirection::Above),
        };
    }

    PreferredNewTrackPlacement {
        insert_index,
        insert_position: Some(direction),
    }
}
