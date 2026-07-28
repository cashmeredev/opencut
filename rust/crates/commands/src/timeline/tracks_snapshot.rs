use scene::{Project, SceneTracks};

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct TracksSnapshotCommand {
    before: SceneTracks,
    after: SceneTracks,
}

impl TracksSnapshotCommand {
    pub fn new(before: SceneTracks, after: SceneTracks) -> Self {
        Self { before, after }
    }
}

impl Command for TracksSnapshotCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        if let Some(scene) = active_scene_mut(project) {
            scene.tracks = self.after.clone();
        }
        Ok(None)
    }

    fn undo(&mut self, project: &mut Project) {
        if let Some(scene) = active_scene_mut(project) {
            scene.tracks = self.before.clone();
        }
    }
}
