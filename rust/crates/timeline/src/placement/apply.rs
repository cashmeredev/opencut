use scene::{Element, SceneTracks, Track};

use super::track_factory::build_empty_track;
use super::{PlacementResult, ordered_tracks};
use crate::track_element_update::update_track_in_scene_tracks;
use crate::types::TrackType;

pub struct PlacementApplied {
    pub updated_tracks: SceneTracks,
    pub target_track_id: String,
}

pub struct ApplyPlacementParams {
    pub tracks: SceneTracks,
    pub placement_result: PlacementResult,
    pub elements: Vec<Element>,
    pub new_track_id: String,
    pub new_track_insert_index_override: Option<usize>,
}

pub fn apply_placement(params: &ApplyPlacementParams) -> Option<PlacementApplied> {
    let tracks = &params.tracks;
    match &params.placement_result {
        PlacementResult::ExistingTrack { track_index, .. } => {
            let ordered = ordered_tracks(tracks);
            let target_track = ordered.get(*track_index)?;

            let updated_tracks =
                update_track_in_scene_tracks(tracks, target_track.id(), |track| {
                    let mut next = track.clone();
                    next.elements_mut().extend(params.elements.iter().cloned());
                    next
                });

            Some(PlacementApplied {
                updated_tracks,
                target_track_id: target_track.id().to_string(),
            })
        }
        PlacementResult::NewTrack {
            track_type,
            insert_index,
            ..
        } => {
            let new_track_id = params.new_track_id.clone();
            let insert_index = params
                .new_track_insert_index_override
                .unwrap_or(*insert_index);
            let mut new_track = build_empty_track(new_track_id.clone(), *track_type, None);
            *new_track.elements_mut() = params.elements.clone();

            let updated_tracks = if *track_type == TrackType::Audio {
                SceneTracks {
                    audio: insert_into_audio_tracks(tracks, insert_index, new_track),
                    ..tracks.clone()
                }
            } else {
                SceneTracks {
                    overlay: insert_into_overlay_tracks(tracks, insert_index, new_track),
                    ..tracks.clone()
                }
            };

            Some(PlacementApplied {
                updated_tracks,
                target_track_id: new_track_id,
            })
        }
    }
}

fn insert_into_overlay_tracks(
    tracks: &SceneTracks,
    insert_index: usize,
    track: Track,
) -> Vec<Track> {
    let normalized = insert_index.min(tracks.overlay.len());
    let mut next = tracks.overlay.clone();
    next.insert(normalized, track);
    next
}

fn insert_into_audio_tracks(tracks: &SceneTracks, insert_index: usize, track: Track) -> Vec<Track> {
    let audio_index = insert_index
        .saturating_sub(tracks.overlay.len() + 1)
        .min(tracks.audio.len());
    let mut next = tracks.audio.clone();
    next.insert(audio_index, track);
    next
}

