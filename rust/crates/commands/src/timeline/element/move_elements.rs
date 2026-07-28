use scene::{Element, Project, SceneTracks, Track};
use time::MediaTime;
use timeline::TrackType;
use timeline::placement::build_empty_track;

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;
use crate::tracks::{all_tracks_mut, find_track};

#[derive(Clone, Debug, PartialEq)]
pub struct PlannedTrackCreation {
    pub id: String,
    pub track_type: TrackType,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlannedElementMove {
    pub source_track_id: String,
    pub target_track_id: String,
    pub element_id: String,
    pub new_start_time: MediaTime,
}

fn insert_track_at_display_index(
    tracks: &mut SceneTracks,
    track: Track,
    insert_index: usize,
) {
    if matches!(track, Track::Audio { .. }) {
        let audio_insert_index = insert_index
            .saturating_sub(tracks.overlay.len() + 1)
            .min(tracks.audio.len());
        tracks.audio.insert(audio_insert_index, track);
        return;
    }

    let overlay_insert_index = insert_index.min(tracks.overlay.len());
    tracks.overlay.insert(overlay_insert_index, track);
}

fn validate_compatibility(element: &Element, track: &Track) -> Result<(), CommandError> {
    let compatibility = timeline::placement::validate_element_track_compatibility(
        timeline::ElementType::of_element(element),
        timeline::TrackType::of_track(track),
    );
    if compatibility.is_valid {
        return Ok(());
    }
    Err(CommandError::IncompatiblePlacement(
        compatibility.error_message.unwrap_or_default(),
    ))
}

pub struct MoveElementCommand {
    moves: Vec<PlannedElementMove>,
    create_tracks: Vec<PlannedTrackCreation>,
    saved_state: Option<SceneTracks>,
}

impl MoveElementCommand {
    pub fn new(moves: Vec<PlannedElementMove>, create_tracks: Vec<PlannedTrackCreation>) -> Self {
        Self {
            moves,
            create_tracks,
            saved_state: None,
        }
    }
}

impl Command for MoveElementCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        let saved_state = scene.tracks.clone();
        self.saved_state = Some(saved_state.clone());

        let mut create_tracks = self.create_tracks.clone();
        create_tracks.sort_by_key(|creation| creation.index);
        for creation in &create_tracks {
            let track = build_empty_track(creation.id.clone(), creation.track_type, None);
            insert_track_at_display_index(&mut scene.tracks, track, creation.index);
        }

        let mut moved_elements = Vec::new();
        for planned_move in &self.moves {
            let source_track = find_track(&saved_state, &planned_move.source_track_id);
            let source_element = source_track.and_then(|track| {
                track
                    .elements()
                    .iter()
                    .find(|element| element.base().id == planned_move.element_id)
            });
            let Some(source_element) = source_element else {
                return Err(CommandError::SourceNotFound);
            };

            let Some(target_track) = find_track(&scene.tracks, &planned_move.target_track_id)
            else {
                return Err(CommandError::TargetTrackNotFound);
            };

            validate_compatibility(source_element, target_track)?;

            let mut moved = source_element.clone();
            moved.base_mut().start_time = planned_move.new_start_time;
            moved_elements.push((planned_move.target_track_id.clone(), moved));
        }

        let moved_ids: Vec<&str> = self
            .moves
            .iter()
            .map(|planned_move| planned_move.element_id.as_str())
            .collect();

        for track in all_tracks_mut(&mut scene.tracks) {
            let track_id = track.id().to_string();
            let mut elements = std::mem::take(track.elements_mut());
            elements.retain(|element| !moved_ids.contains(&element.base().id.as_str()));
            elements.extend(
                moved_elements
                    .iter()
                    .filter(|(target_track_id, _)| *target_track_id == track_id)
                    .map(|(_, element)| element.clone()),
            );
            *track.elements_mut() = elements;
        }

        Ok(Some(CommandResult::element_selection(
            self.moves
                .iter()
                .map(|planned_move| scene::ElementRef {
                    track_id: planned_move.target_track_id.clone(),
                    element_id: planned_move.element_id.clone(),
                })
                .collect(),
        )))
    }

    fn undo(&mut self, project: &mut Project) {
        let Some(saved) = &self.saved_state else {
            return;
        };
        if let Some(scene) = active_scene_mut(project) {
            scene.tracks = saved.clone();
        }
    }
}
