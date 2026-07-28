use scene::{ElementRef, Project};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("source track or element not found")]
    SourceNotFound,
    #[error("target track not found")]
    TargetTrackNotFound,
    #[error("{0}")]
    IncompatiblePlacement(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectionPatch {
    pub selected_elements: Vec<ElementRef>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommandResult {
    pub selection: Option<SelectionPatch>,
}

impl CommandResult {
    pub fn element_selection(selected_elements: Vec<ElementRef>) -> Self {
        Self {
            selection: Some(SelectionPatch { selected_elements }),
        }
    }
}

pub trait Command {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError>;

    fn undo(&mut self, project: &mut Project);

    fn redo(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        self.execute(project)
    }
}
