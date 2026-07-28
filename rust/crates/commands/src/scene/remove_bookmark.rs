use scene::Project;
use time::MediaTime;

use crate::bookmarks::{get_frame_time, remove_bookmark_from_array};
use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct RemoveBookmarkCommand {
    time: MediaTime,
    frame_time: MediaTime,
    saved_bookmarks: Option<Vec<scene::Bookmark>>,
}

impl RemoveBookmarkCommand {
    pub fn new(time: MediaTime) -> Self {
        Self {
            time,
            frame_time: MediaTime::ZERO,
            saved_bookmarks: None,
        }
    }
}

impl Command for RemoveBookmarkCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let fps = project.settings.fps;
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_bookmarks = Some(scene.bookmarks.clone());
        self.frame_time = get_frame_time(self.time, fps);

        let updated = remove_bookmark_from_array(&scene.bookmarks, self.frame_time);
        if updated.len() == scene.bookmarks.len() {
            return Ok(None);
        }

        scene.bookmarks = updated;
        Ok(None)
    }

    fn undo(&mut self, project: &mut Project) {
        let Some(saved) = &self.saved_bookmarks else {
            return;
        };
        if let Some(scene) = active_scene_mut(project) {
            scene.bookmarks = saved.clone();
        }
    }
}
