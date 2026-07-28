use scene::{Project, Scene};

use crate::command::{Command, CommandError, CommandResult};

pub struct CreateSceneCommand {
    name: String,
    is_main: bool,
    saved_scenes: Option<Vec<Scene>>,
    created_scene_id: Option<String>,
}

impl CreateSceneCommand {
    pub fn new(name: impl Into<String>, is_main: bool) -> Self {
        Self {
            name: name.into(),
            is_main,
            saved_scenes: None,
            created_scene_id: None,
        }
    }

    pub fn scene_id(&self) -> Option<&str> {
        self.created_scene_id.as_deref()
    }
}

impl Command for CreateSceneCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        self.saved_scenes = Some(project.scenes.clone());

        let created = timeline::scenes::build_default_scene(&self.name, self.is_main);
        self.created_scene_id = Some(created.id.clone());
        project.scenes.push(created);
        Ok(None)
    }

    fn undo(&mut self, project: &mut Project) {
        let Some(saved) = &self.saved_scenes else {
            return;
        };
        project.scenes = saved.clone();
    }
}
