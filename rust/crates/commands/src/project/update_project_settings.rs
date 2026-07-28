use scene::{Background, CanvasSize, CanvasSizeMode, Project, ProjectSettings};
use time::FrameRate;

use crate::command::{Command, CommandError, CommandResult};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettingsPatch {
    pub fps: Option<FrameRate>,
    pub canvas_size: Option<CanvasSize>,
    pub canvas_size_mode: Option<Option<CanvasSizeMode>>,
    pub last_custom_canvas_size: Option<Option<CanvasSize>>,
    pub original_canvas_size: Option<Option<CanvasSize>>,
    pub background: Option<Background>,
}

impl ProjectSettingsPatch {
    pub fn apply_to(&self, settings: &mut ProjectSettings) {
        if let Some(fps) = self.fps {
            settings.fps = fps;
        }
        if let Some(canvas_size) = self.canvas_size {
            settings.canvas_size = canvas_size;
        }
        if let Some(canvas_size_mode) = &self.canvas_size_mode {
            settings.canvas_size_mode = *canvas_size_mode;
        }
        if let Some(last_custom_canvas_size) = &self.last_custom_canvas_size {
            settings.last_custom_canvas_size = *last_custom_canvas_size;
        }
        if let Some(original_canvas_size) = &self.original_canvas_size {
            settings.original_canvas_size = *original_canvas_size;
        }
        if let Some(background) = &self.background {
            settings.background = background.clone();
        }
    }
}

pub struct UpdateProjectSettingsCommand {
    updates: ProjectSettingsPatch,
    saved_settings: Option<ProjectSettings>,
    saved_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UpdateProjectSettingsCommand {
    pub fn new(updates: ProjectSettingsPatch) -> Self {
        Self {
            updates,
            saved_settings: None,
            saved_updated_at: None,
        }
    }
}

impl Command for UpdateProjectSettingsCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        self.saved_settings = Some(project.settings.clone());
        self.saved_updated_at = Some(project.metadata.updated_at);

        self.updates.apply_to(&mut project.settings);
        project.metadata.updated_at = chrono::Utc::now();
        Ok(None)
    }

    fn undo(&mut self, project: &mut Project) {
        let (Some(saved_settings), Some(saved_updated_at)) =
            (&self.saved_settings, self.saved_updated_at)
        else {
            return;
        };
        project.settings = saved_settings.clone();
        project.metadata.updated_at = saved_updated_at;
    }
}
