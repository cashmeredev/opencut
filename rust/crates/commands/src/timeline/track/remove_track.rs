use scene::{Project, SceneTracks};

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct RemoveTrackCommand {
    track_id: String,
    saved_state: Option<SceneTracks>,
}

impl RemoveTrackCommand {
    pub fn new(track_id: impl Into<String>) -> Self {
        Self {
            track_id: track_id.into(),
            saved_state: None,
        }
    }
}

impl Command for RemoveTrackCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());
        scene.tracks.overlay.retain(|track| track.id() != self.track_id);
        scene.tracks.audio.retain(|track| track.id() != self.track_id);
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
