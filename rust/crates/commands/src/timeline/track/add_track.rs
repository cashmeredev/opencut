use scene::{Project, SceneTracks};
use timeline::placement::{build_empty_track, get_default_insert_index_for_track};
use timeline::TrackType;
use uuid::Uuid;

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct AddTrackCommand {
    track_type: TrackType,
    index: Option<usize>,
    track_id: String,
    saved_state: Option<SceneTracks>,
}

impl AddTrackCommand {
    pub fn new(track_type: TrackType, index: Option<usize>) -> Self {
        Self {
            track_type,
            index,
            track_id: Uuid::new_v4().to_string(),
            saved_state: None,
        }
    }

    pub fn track_id(&self) -> &str {
        &self.track_id
    }
}

impl Command for AddTrackCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());

        let insert_index = self.index.unwrap_or_else(|| {
            get_default_insert_index_for_track(&scene.tracks, self.track_type)
        });

        let new_track = build_empty_track(self.track_id.clone(), self.track_type, None);
        if self.track_type == TrackType::Audio {
            let audio_insert_index = insert_index
                .saturating_sub(scene.tracks.overlay.len() + 1);
            scene
                .tracks
                .audio
                .insert(audio_insert_index.min(scene.tracks.audio.len()), new_track);
        } else {
            let overlay_insert_index = insert_index.min(scene.tracks.overlay.len());
            scene.tracks.overlay.insert(overlay_insert_index, new_track);
        }
        Ok(None)
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
