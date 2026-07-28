use scene::{ElementRef, Project, SceneTracks, Track};

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

fn remove_track_elements(track: &mut Track, elements: &[ElementRef]) {
    if !elements.iter().any(|target| target.track_id == track.id()) {
        return;
    }
    track.elements_mut().retain(|element| {
        !elements
            .iter()
            .any(|target| target.track_id == track.id() && target.element_id == element.base().id)
    });
}

fn remove_elements_from_tracks(tracks: &mut SceneTracks, elements: &[ElementRef]) {
    for track in &mut tracks.overlay {
        remove_track_elements(track, elements);
    }
    remove_track_elements(&mut tracks.main, elements);
    for track in &mut tracks.audio {
        remove_track_elements(track, elements);
    }
}

pub struct DeleteElementsCommand {
    elements: Vec<ElementRef>,
    saved_state: Option<SceneTracks>,
}

impl DeleteElementsCommand {
    pub fn new(elements: Vec<ElementRef>) -> Self {
        Self {
            elements,
            saved_state: None,
        }
    }
}

impl Command for DeleteElementsCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());
        remove_elements_from_tracks(&mut scene.tracks, &self.elements);

        Ok(Some(CommandResult::element_selection(Vec::new())))
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
