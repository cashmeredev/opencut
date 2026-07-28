use scene::{Project, SceneTracks, Track};

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;
use crate::tracks::find_track_mut;

pub struct ToggleTrackMuteCommand {
    track_id: String,
    saved_state: Option<SceneTracks>,
}

impl ToggleTrackMuteCommand {
    pub fn new(track_id: impl Into<String>) -> Self {
        Self {
            track_id: track_id.into(),
            saved_state: None,
        }
    }
}

impl Command for ToggleTrackMuteCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());

        match find_track_mut(&mut scene.tracks, &self.track_id) {
            Some(Track::Video { muted, .. }) | Some(Track::Audio { muted, .. }) => {
                *muted = !*muted;
            }
            _ => {}
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
