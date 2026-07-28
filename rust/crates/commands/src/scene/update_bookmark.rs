use scene::Project;
use time::MediaTime;

use crate::bookmarks::{BookmarkPatch, get_frame_time, update_bookmark_in_array};
use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct UpdateBookmarkCommand {
    time: MediaTime,
    updates: BookmarkPatch,
    saved_bookmarks: Option<Vec<scene::Bookmark>>,
}

impl UpdateBookmarkCommand {
    pub fn new(time: MediaTime, updates: BookmarkPatch) -> Self {
        Self {
            time,
            updates,
            saved_bookmarks: None,
        }
    }
}

impl Command for UpdateBookmarkCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let fps = project.settings.fps;
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_bookmarks = Some(scene.bookmarks.clone());
        let frame_time = get_frame_time(self.time, fps);
        scene.bookmarks =
            update_bookmark_in_array(&scene.bookmarks, frame_time, &self.updates);
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
