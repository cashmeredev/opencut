use scene::{ElementRef, Project, SceneTracks};
use timeline::update_pipeline::{ElementPatch, ElementUpdateContext, apply_element_update};

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;
use crate::tracks::find_track_mut;

#[derive(Clone, Debug, PartialEq)]
pub struct ElementUpdate {
    pub track_id: String,
    pub element_id: String,
    pub patch: ElementPatch,
}

pub struct UpdateElementsCommand {
    updates: Vec<ElementUpdate>,
    saved_state: Option<SceneTracks>,
}

impl UpdateElementsCommand {
    pub fn new(updates: Vec<ElementUpdate>) -> Self {
        Self {
            updates,
            saved_state: None,
        }
    }
}

impl Command for UpdateElementsCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());

        for update in &self.updates {
            let context_tracks = scene.tracks.clone();
            let context = ElementUpdateContext {
                tracks: &context_tracks,
                track_id: &update.track_id,
            };
            let Some(track) = find_track_mut(&mut scene.tracks, &update.track_id) else {
                continue;
            };
            let Some(element) = track
                .elements_mut()
                .iter_mut()
                .find(|element| element.base().id == update.element_id)
            else {
                continue;
            };

            let current = element.clone();
            *element = apply_element_update(current, update.patch.clone(), &context);
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
