use scene::{Project, Scene};

use crate::command::{Command, CommandError, CommandResult};

pub struct DeleteSceneCommand {
    scene_id: String,
    saved_scenes: Option<Vec<Scene>>,
    saved_current_scene_id: Option<String>,
}

impl DeleteSceneCommand {
    pub fn new(scene_id: impl Into<String>) -> Self {
        Self {
            scene_id: scene_id.into(),
            saved_scenes: None,
            saved_current_scene_id: None,
        }
    }
}

impl Command for DeleteSceneCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(deleted) = project.scenes.iter().find(|scene| scene.id == self.scene_id)
        else {
            return Ok(None);
        };
        if deleted.is_main {
            return Ok(None);
        }

        self.saved_scenes = Some(project.scenes.clone());
        self.saved_current_scene_id = Some(project.current_scene_id.clone());

        project.scenes.retain(|scene| scene.id != self.scene_id);

        if project.current_scene_id == self.scene_id {
            let fallback_id = project
                .scenes
                .iter()
                .find(|scene| scene.is_main)
                .map(|scene| scene.id.clone());
            if let Some(fallback_id) = fallback_id {
                project.current_scene_id = fallback_id;
            }
        }
        Ok(None)
    }

    fn undo(&mut self, project: &mut Project) {
        let (Some(saved_scenes), Some(saved_current_scene_id)) =
            (&self.saved_scenes, &self.saved_current_scene_id)
        else {
            return;
        };
        project.scenes = saved_scenes.clone();
        project.current_scene_id = saved_current_scene_id.clone();
    }
}
