mod common;

use commands::{
    BatchCommand, Command, InsertElementCommand, InsertElementPlacement, RetainSide,
    ShiftSplitRemainderCommand, SplitElementsCommand,
};
use common::{
    FREEZE_SECONDS, SPLIT_SECONDS, VIDEO_SECONDS, build_freeze_image_element, seconds,
    seed_project_with_video,
};
use scene::{Element, ElementRef, Project};
use time::MediaTime;

fn build_freeze_batch(track_id: &str) -> BatchCommand {
    let split = SplitElementsCommand::new(
        vec![ElementRef {
            track_id: track_id.to_string(),
            element_id: "video-1".to_string(),
        }],
        seconds(SPLIT_SECONDS),
        RetainSide::Both,
    );
    let right_side = split.right_side_handle();
    let shift = ShiftSplitRemainderCommand::new(
        right_side,
        seconds(SPLIT_SECONDS) + seconds(FREEZE_SECONDS),
    );
    let insert = InsertElementCommand::new(
        build_freeze_image_element(),
        InsertElementPlacement::Explicit {
            track_id: track_id.to_string(),
        },
    );
    BatchCommand::new(vec![Box::new(split), Box::new(shift), Box::new(insert)])
}

fn sorted_main_elements(project: &Project) -> Vec<Element> {
    let scene = project
        .scenes
        .iter()
        .find(|scene| scene.id == project.current_scene_id)
        .expect("active scene");
    let mut elements = scene.tracks.main.elements().to_vec();
    elements.sort_by_key(|element| element.base().start_time);
    elements
}

fn element_type_name(element: &Element) -> &'static str {
    match element {
        Element::Video(_) => "video",
        Element::Image(_) => "image",
        Element::Text(_) => "text",
        Element::Audio(_) => "audio",
        Element::Sticker(_) => "sticker",
        Element::Graphic(_) => "graphic",
        Element::Effect(_) => "effect",
    }
}

#[test]
fn splits_shifts_right_side_and_inserts_into_gap() {
    let (mut project, track_id) = seed_project_with_video();

    build_freeze_batch(&track_id)
        .execute(&mut project)
        .expect("execute");

    let elements = sorted_main_elements(&project);
    let type_names: Vec<&str> = elements.iter().map(element_type_name).collect();
    assert_eq!(type_names, ["video", "image", "video"]);

    let [left, image, right] = &elements[..] else {
        panic!("expected three elements");
    };
    assert_eq!(left.base().start_time, MediaTime::ZERO);
    assert_eq!(left.base().duration, seconds(SPLIT_SECONDS));
    assert_eq!(image.base().start_time, seconds(SPLIT_SECONDS));
    assert_eq!(image.base().duration, seconds(FREEZE_SECONDS));
    assert_eq!(
        right.base().start_time,
        seconds(SPLIT_SECONDS) + seconds(FREEZE_SECONDS)
    );
    assert_eq!(
        right.base().duration,
        seconds(VIDEO_SECONDS - SPLIT_SECONDS)
    );
}

#[test]
fn undo_restores_original_element_as_single_step() {
    let (mut project, track_id) = seed_project_with_video();
    let mut batch = build_freeze_batch(&track_id);

    batch.execute(&mut project).expect("execute");
    batch.undo(&mut project);

    let scene = project
        .scenes
        .iter()
        .find(|scene| scene.id == project.current_scene_id)
        .expect("active scene");
    let elements = scene.tracks.main.elements();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].base().id, "video-1");
    assert_eq!(elements[0].base().start_time, MediaTime::ZERO);
    assert_eq!(elements[0].base().duration, seconds(VIDEO_SECONDS));
}

#[test]
fn redo_after_undo_reapplies_shift_to_resplit_elements() {
    let (mut project, track_id) = seed_project_with_video();
    let mut batch = build_freeze_batch(&track_id);

    batch.execute(&mut project).expect("execute");
    batch.undo(&mut project);
    batch.redo(&mut project).expect("redo");

    let elements = sorted_main_elements(&project);
    let type_names: Vec<&str> = elements.iter().map(element_type_name).collect();
    assert_eq!(type_names, ["video", "image", "video"]);

    let right = elements.last().expect("right element");
    assert!(matches!(right, Element::Video(_)));
    assert_eq!(
        right.base().start_time,
        seconds(SPLIT_SECONDS) + seconds(FREEZE_SECONDS)
    );
}

#[test]
fn no_op_when_split_produces_no_right_side() {
    let (mut project, track_id) = seed_project_with_video();

    let split = SplitElementsCommand::new(
        vec![ElementRef {
            track_id: track_id.to_string(),
            element_id: "video-1".to_string(),
        }],
        MediaTime::ZERO,
        RetainSide::Both,
    );
    let right_side = split.right_side_handle();
    let shift = ShiftSplitRemainderCommand::new(right_side, seconds(FREEZE_SECONDS));
    let mut batch = BatchCommand::new(vec![Box::new(split), Box::new(shift)]);

    batch.execute(&mut project).expect("execute");
    batch.undo(&mut project);

    let scene = project
        .scenes
        .iter()
        .find(|scene| scene.id == project.current_scene_id)
        .expect("active scene");
    assert_eq!(scene.tracks.main.elements().len(), 1);
}
