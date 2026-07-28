use std::cell::RefCell;
use std::rc::Rc;

use scene::{ElementRef, Project, SceneTracks};
use time::MediaTime;
pub use timeline::split::RetainSide;

use crate::command::{Command, CommandError, CommandResult};
use crate::project_ext::active_scene_mut;

pub struct SplitElementsCommand {
    elements: Vec<ElementRef>,
    split_time: MediaTime,
    retain_side: RetainSide,
    saved_state: Option<SceneTracks>,
    right_side: Rc<RefCell<Vec<ElementRef>>>,
}

impl SplitElementsCommand {
    pub fn new(elements: Vec<ElementRef>, split_time: MediaTime, retain_side: RetainSide) -> Self {
        Self {
            elements,
            split_time,
            retain_side,
            saved_state: None,
            right_side: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn right_side_handle(&self) -> Rc<RefCell<Vec<ElementRef>>> {
        Rc::clone(&self.right_side)
    }

    pub fn right_side_elements(&self) -> Vec<ElementRef> {
        self.right_side.borrow().clone()
    }
}

impl Command for SplitElementsCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(scene) = active_scene_mut(project) else {
            return Ok(None);
        };

        self.saved_state = Some(scene.tracks.clone());

        let result = timeline::split::split_elements(
            &scene.tracks,
            &self.elements,
            self.split_time,
            self.retain_side,
        );
        scene.tracks = result.tracks;

        *self.right_side.borrow_mut() = result.right_side_elements;

        let right_side = self.right_side.borrow();
        if right_side.is_empty() {
            return Ok(None);
        }
        Ok(Some(CommandResult::element_selection(right_side.clone())))
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
