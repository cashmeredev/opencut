use scene::Project;

use crate::command::{Command, CommandError, CommandResult};

pub struct RenameSceneCommand {
    scene_id: String,
    new_name: String,
    saved_name: Option<String>,
    saved_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl RenameSceneCommand {
    pub fn new(scene_id: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self {
            scene_id: scene_id.into(),
            new_name: new_name.into(),
            saved_name: None,
            saved_updated_at: None,
        }
    }
}

impl Command for RenameSceneCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = project
            .scenes
            .iter_mut()
            .find(|scene| scene.id == self.scene_id)
        else {
            return Ok(None);
        };

        self.saved_name = Some(scene.name.clone());
        self.saved_updated_at = Some(scene.updated_at);
        scene.name = self.new_name.clone();
        scene.updated_at = chrono::Utc::now();
        Ok(None)
    }

    fn undo(&mut self, project: &mut Project) {
        let (Some(saved_name), Some(saved_updated_at)) =
            (&self.saved_name, self.saved_updated_at)
        else {
            return;
        };
        if let Some(scene) = project
            .scenes
            .iter_mut()
            .find(|scene| scene.id == self.scene_id)
        {
            scene.name = saved_name.clone();
            scene.updated_at = saved_updated_at;
        }
    }
}
