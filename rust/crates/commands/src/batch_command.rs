use scene::Project;

use crate::command::{Command, CommandError, CommandResult};

pub struct BatchCommand {
    commands: Vec<Box<dyn Command>>,
}

impl BatchCommand {
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self { commands }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Command for BatchCommand {
    fn execute(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let mut latest_selection_result = None;

        for command in &mut self.commands {
            let result = command.execute(project)?;
            if result.as_ref().and_then(|r| r.selection.as_ref()).is_some() {
                latest_selection_result = result;
            }
        }

        Ok(latest_selection_result)
    }

    fn undo(&mut self, project: &mut Project) {
        for command in self.commands.iter_mut().rev() {
            command.undo(project);
        }
    }

    fn redo(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<CommandResult>, CommandError> {
        let mut latest_selection_result = None;

        for command in &mut self.commands {
            let result = command.redo(project)?;
            if result.as_ref().and_then(|r| r.selection.as_ref()).is_some() {
                latest_selection_result = result;
            }
        }

        Ok(latest_selection_result)
    }
}
