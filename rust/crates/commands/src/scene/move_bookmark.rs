use scene::Project;
use time::MediaTime;

use crate::bookmarks::{get_frame_time, move_bookmark_in_array};
use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct MoveBookmarkCommand {
    from_time: MediaTime,
    to_time: MediaTime,
    saved_bookmarks: Option<Vec<scene::Bookmark>>,
}

impl MoveBookmarkCommand {
    pub fn new(from_time: MediaTime, to_time: MediaTime) -> Self {
        Self {
            from_time,
            to_time,
            saved_bookmarks: None,
        }
    }
}

impl Command for MoveBookmarkCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let fps = project.settings.fps;
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_bookmarks = Some(scene.bookmarks.clone());
        let from_frame_time = get_frame_time(self.from_time, fps);
        let to_frame_time = get_frame_time(self.to_time, fps);
        scene.bookmarks =
            move_bookmark_in_array(&scene.bookmarks, from_frame_time, to_frame_time);
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
