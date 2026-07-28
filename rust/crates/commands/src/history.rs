use scene::Project;

use crate::command::{Command, CommandError, CommandResult};

#[derive(Default)]
pub struct CommandHistory {
    history: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let result = command.execute(project)?;
        self.history.push(command);
        self.redo_stack.clear();
        Ok(result)
    }

    pub fn push(&mut self, command: Box<dyn Command>) {
        self.history.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, project: &mut Project) {
        let Some(mut command) = self.history.pop() else {
            return;
        };
        command.undo(project);
        self.redo_stack.push(command);
    }

    pub fn redo(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let Some(mut command) = self.redo_stack.pop() else {
            return Ok(None);
        };
        let result = command.redo(project)?;
        self.history.push(command);
        Ok(result)
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.redo_stack.clear();
    }
}
