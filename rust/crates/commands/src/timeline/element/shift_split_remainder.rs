use std::cell::RefCell;
use std::rc::Rc;

use scene::{ElementRef, Project};
use time::MediaTime;

use crate::command::{Command, CommandError, CommandResult};
use crate::timeline::element::move_elements::{MoveElementCommand, PlannedElementMove};

pub struct ShiftSplitRemainderCommand {
    right_side: Rc<RefCell<Vec<ElementRef>>>,
    new_start_time: MediaTime,
    inner: Option<MoveElementCommand>,
}

impl ShiftSplitRemainderCommand {
    pub fn new(right_side: Rc<RefCell<Vec<ElementRef>>>, new_start_time: MediaTime) -> Self {
        Self {
            right_side,
            new_start_time,
            inner: None,
        }
    }
}

impl Command for ShiftSplitRemainderCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let right_side = self.right_side.borrow().clone();
        if right_side.is_empty() {
            return Ok(None);
        }

        let mut inner = MoveElementCommand::new(
            right_side
                .iter()
                .map(|element| PlannedElementMove {
                    source_track_id: element.track_id.clone(),
                    target_track_id: element.track_id.clone(),
                    element_id: element.element_id.clone(),
                    new_start_time: self.new_start_time,
                })
                .collect(),
            Vec::new(),
        );
        let result = inner.execute(project)?;
        self.inner = Some(inner);
        Ok(result)
    }

    fn undo(&mut self, project: &mut Project) {
        if let Some(inner) = &mut self.inner {
            inner.undo(project);
        }
    }
}
