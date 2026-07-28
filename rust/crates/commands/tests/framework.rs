mod common;

use std::cell::RefCell;
use std::rc::Rc;

use commands::{
    BatchCommand, Command, CommandError, CommandHistory, CommandResult, DeleteElementsCommand,
    PreviewTracker,
};
use common::{VIDEO_SECONDS, seconds, seed_project_with_video};
use scene::{ElementRef, Project};

struct RecordingCommand {
    label: &'static str,
    log: Rc<RefCell<Vec<&'static str>>>,
    selection: Option<Vec<ElementRef>>,
}

impl RecordingCommand {
    fn new(label: &'static str, log: &Rc<RefCell<Vec<&'static str>>>) -> Self {
        Self {
            label,
            log: Rc::clone(log),
            selection: None,
        }
    }

    fn with_selection(mut self, selection: Vec<ElementRef>) -> Self {
        self.selection = Some(selection);
        self
    }
}

impl Command for RecordingCommand {
    fn execute(&mut self, _project: &mut Project) -> Result<Option<CommandResult>, CommandError> {
        self.log.borrow_mut().push(self.label);
        Ok(self
            .selection
            .clone()
            .map(CommandResult::element_selection))
    }

    fn undo(&mut self, _project: &mut Project) {
        self.log.borrow_mut().push(self.label);
    }
}

#[test]
fn batch_executes_in_order_and_undoes_in_reverse() {
    let (mut project, _) = seed_project_with_video();
    let log = Rc::new(RefCell::new(Vec::new()));

    let mut batch = BatchCommand::new(vec![
        Box::new(RecordingCommand::new("first", &log)),
        Box::new(RecordingCommand::new("second", &log)),
        Box::new(RecordingCommand::new("third", &log)),
    ]);

    batch.execute(&mut project).expect("execute");
    assert_eq!(*log.borrow(), vec!["first", "second", "third"]);

    log.borrow_mut().clear();
    batch.undo(&mut project);
    assert_eq!(*log.borrow(), vec!["third", "second", "first"]);
}

#[test]
fn batch_returns_latest_selection_result() {
    let (mut project, _) = seed_project_with_video();
    let log = Rc::new(RefCell::new(Vec::new()));

    let selected = vec![ElementRef {
        track_id: "main-track-1".to_string(),
        element_id: "video-1".to_string(),
    }];
    let mut batch = BatchCommand::new(vec![
        Box::new(
            RecordingCommand::new("selecting", &log).with_selection(selected.clone()),
        ),
        Box::new(RecordingCommand::new("plain", &log)),
    ]);

    let result = batch.execute(&mut project).expect("execute");
    let selection = result
        .and_then(|result| result.selection)
        .expect("selection result");
    assert_eq!(selection.selected_elements, selected);
}

#[test]
fn history_undo_redo_roundtrip() {
    let (mut project, track_id) = seed_project_with_video();
    let mut history = CommandHistory::new();

    assert!(!history.can_undo());
    assert!(!history.can_redo());

    history
        .execute(
            Box::new(DeleteElementsCommand::new(vec![ElementRef {
                track_id: track_id.clone(),
                element_id: "video-1".to_string(),
            }])),
            &mut project,
        )
        .expect("execute");

    let scene = common::active_scene;
    assert!(scene(&project).tracks.main.elements().is_empty());
    assert!(history.can_undo());
    assert!(!history.can_redo());

    history.undo(&mut project);
    assert_eq!(scene(&project).tracks.main.elements().len(), 1);
    assert!(!history.can_undo());
    assert!(history.can_redo());

    history.redo(&mut project).expect("redo");
    assert!(scene(&project).tracks.main.elements().is_empty());
    assert!(history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn history_new_execute_clears_redo_stack() {
    let (mut project, track_id) = seed_project_with_video();
    let mut history = CommandHistory::new();

    let delete = || {
        Box::new(DeleteElementsCommand::new(vec![ElementRef {
            track_id: track_id.clone(),
            element_id: "video-1".to_string(),
        }])) as Box<dyn Command>
    };

    history.execute(delete(), &mut project).expect("execute");
    history.undo(&mut project);
    assert!(history.can_redo());

    history.execute(delete(), &mut project).expect("execute");
    assert!(!history.can_redo());

    history.clear();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn preview_tracker_keeps_first_snapshot_until_end() {
    let mut tracker = PreviewTracker::new();
    assert!(!tracker.is_active());
    assert_eq!(tracker.end(), None);

    tracker.begin(&seconds(VIDEO_SECONDS));
    tracker.begin(&seconds(1.0));
    assert!(tracker.is_active());
    assert_eq!(tracker.snapshot(), Some(&seconds(VIDEO_SECONDS)));

    assert_eq!(tracker.end(), Some(seconds(VIDEO_SECONDS)));
    assert!(!tracker.is_active());
    assert_eq!(tracker.snapshot(), None);
}
