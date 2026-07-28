use scene::{
    BaseTimelineElement, Element, ElementRef, ImageElement, ParamValue, ParamValues, SceneTracks,
    Track, VideoElement,
};
use time::{FrameRate, MediaTime};
use timeline::resize::{
    GroupResizeMember, GroupResizePushTarget, GroupResizeResult, GroupResizeUpdate, ResizeSide,
    build_resize_members, compute_group_resize,
};
use timeline::track_element_update::update_element_in_scene_tracks;

const FPS: FrameRate = FrameRate::new(30, 1);

fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).unwrap()
}

fn build_push_target(element_id: &str, start_time: MediaTime, duration: MediaTime) -> GroupResizePushTarget {
    GroupResizePushTarget {
        track_id: "track-1".to_string(),
        element_id: element_id.to_string(),
        start_time,
        duration,
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
    }
}

fn build_static_member(
    start_time: MediaTime,
    duration: MediaTime,
    left_neighbor_bound: Option<MediaTime>,
    right_neighbor_bound: Option<MediaTime>,
    right_push_chain: Option<Vec<GroupResizePushTarget>>,
) -> GroupResizeMember {
    GroupResizeMember {
        track_id: "track-1".to_string(),
        element_id: "element-1".to_string(),
        start_time,
        duration,
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: None,
        retime: None,
        left_neighbor_bound,
        right_neighbor_bound,
        right_push_chain,
    }
}

fn build_video_member(
    start_time: MediaTime,
    duration: MediaTime,
    source_duration: MediaTime,
    right_neighbor_bound: Option<MediaTime>,
) -> GroupResizeMember {
    GroupResizeMember {
        track_id: "track-1".to_string(),
        element_id: "element-1".to_string(),
        start_time,
        duration,
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: Some(source_duration),
        retime: None,
        left_neighbor_bound: None,
        right_neighbor_bound,
        right_push_chain: None,
    }
}

fn find_update<'a>(result: &'a GroupResizeResult, element_id: &str) -> Option<&'a GroupResizeUpdate> {
    result
        .updates
        .iter()
        .find(|update| update.element_id == element_id)
}

#[test]
fn extends_into_the_gap_before_pushing_anything() {
    let member = build_static_member(
        MediaTime::ZERO,
        seconds(5.0),
        None,
        Some(seconds(10.0)),
        Some(vec![build_push_target("neighbor-1", seconds(10.0), seconds(10.0))]),
    );

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(3.0), FPS);

    assert_eq!(result.delta_time, seconds(3.0));
    assert_eq!(result.updates.len(), 1);
    assert_eq!(
        find_update(&result, "element-1").unwrap().patch.duration,
        seconds(8.0)
    );
}

#[test]
fn pushes_the_flush_right_neighbor_by_the_overflow() {
    let member = build_static_member(
        MediaTime::ZERO,
        seconds(5.0),
        None,
        Some(seconds(5.0)),
        Some(vec![build_push_target("neighbor-1", seconds(5.0), seconds(10.0))]),
    );

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(3.0), FPS);

    assert_eq!(result.delta_time, seconds(3.0));
    assert_eq!(result.updates.len(), 2);
    assert_eq!(
        find_update(&result, "element-1").unwrap().patch.duration,
        seconds(8.0)
    );
    let push = find_update(&result, "neighbor-1").unwrap();
    assert_eq!(push.patch.start_time, seconds(8.0));
    assert_eq!(push.patch.duration, seconds(10.0));
    assert_eq!(push.patch.trim_start, MediaTime::ZERO);
    assert_eq!(push.patch.trim_end, MediaTime::ZERO);
}

#[test]
fn pushes_a_chain_of_several_neighbors_by_the_same_overflow() {
    let member = build_static_member(
        MediaTime::ZERO,
        seconds(5.0),
        None,
        Some(seconds(5.0)),
        Some(vec![
            build_push_target("neighbor-1", seconds(5.0), seconds(10.0)),
            build_push_target("neighbor-2", seconds(15.0), seconds(5.0)),
            build_push_target("neighbor-3", seconds(25.0), seconds(2.0)),
        ]),
    );

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(4.0), FPS);

    assert_eq!(result.delta_time, seconds(4.0));
    assert_eq!(result.updates.len(), 4);
    assert_eq!(
        find_update(&result, "neighbor-1").unwrap().patch.start_time,
        seconds(9.0)
    );
    assert_eq!(
        find_update(&result, "neighbor-2").unwrap().patch.start_time,
        seconds(19.0)
    );
    assert_eq!(
        find_update(&result, "neighbor-3").unwrap().patch.start_time,
        seconds(29.0)
    );
}

#[test]
fn push_stops_at_the_end_of_the_track() {
    let member = build_static_member(MediaTime::ZERO, seconds(5.0), None, None, Some(Vec::new()));

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(3.0), FPS);

    assert_eq!(result.delta_time, seconds(3.0));
    assert_eq!(result.updates.len(), 1);
    assert_eq!(
        find_update(&result, "element-1").unwrap().patch.duration,
        seconds(8.0)
    );
}

#[test]
fn video_element_clamps_at_its_right_neighbor_instead_of_pushing() {
    let member = build_video_member(
        MediaTime::ZERO,
        seconds(5.0),
        seconds(6.0),
        Some(seconds(5.0)),
    );

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(10.0), FPS);

    assert_eq!(result.delta_time, MediaTime::ZERO);
    assert_eq!(result.updates.len(), 1);
    assert_eq!(
        find_update(&result, "element-1").unwrap().patch.duration,
        seconds(5.0)
    );
}

#[test]
fn video_element_clamps_at_its_source_extent_when_there_is_no_neighbor() {
    let member = build_video_member(MediaTime::ZERO, seconds(5.0), seconds(6.0), None);

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(10.0), FPS);

    assert_eq!(result.delta_time, seconds(1.0));
    assert_eq!(result.updates.len(), 1);
    assert_eq!(
        find_update(&result, "element-1").unwrap().patch.duration,
        seconds(6.0)
    );
}

#[test]
fn static_element_without_a_push_chain_keeps_the_old_neighbor_clamp() {
    let member = build_static_member(MediaTime::ZERO, seconds(5.0), None, Some(seconds(5.0)), None);

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(3.0), FPS);

    assert_eq!(result.delta_time, MediaTime::ZERO);
    assert_eq!(result.updates.len(), 1);
}

#[test]
fn shortening_still_respects_the_one_frame_minimum_duration() {
    let one_frame = MediaTime::from_ticks((time::TICKS_PER_SECOND as f64 / 30.0).round() as i64);
    let member = build_static_member(MediaTime::ZERO, seconds(5.0), None, None, Some(Vec::new()));

    let result = compute_group_resize(&[member], ResizeSide::Right, seconds(-10.0), FPS);

    assert_eq!(result.updates.len(), 1);
    assert_eq!(
        find_update(&result, "element-1").unwrap().patch.duration,
        one_frame
    );
}

#[test]
fn left_edge_resize_still_clamps_at_the_left_neighbor_and_pushes_nothing() {
    let member = build_static_member(
        seconds(5.0),
        seconds(5.0),
        Some(seconds(2.0)),
        None,
        Some(vec![build_push_target("neighbor-1", seconds(10.0), seconds(10.0))]),
    );

    let result = compute_group_resize(&[member], ResizeSide::Left, seconds(-10.0), FPS);

    assert_eq!(result.delta_time, seconds(-3.0));
    assert_eq!(result.updates.len(), 1);
    let update = find_update(&result, "element-1").unwrap();
    assert_eq!(update.patch.start_time, seconds(2.0));
    assert_eq!(update.patch.duration, seconds(8.0));
}

fn visual_params() -> ParamValues {
    ParamValues::from([
        ("transform.positionX".to_string(), ParamValue::Number(0.0)),
        ("transform.positionY".to_string(), ParamValue::Number(0.0)),
        ("transform.scaleX".to_string(), ParamValue::Number(1.0)),
        ("transform.scaleY".to_string(), ParamValue::Number(1.0)),
        ("transform.rotate".to_string(), ParamValue::Number(0.0)),
        ("opacity".to_string(), ParamValue::Number(1.0)),
    ])
}

fn base_element(id: &str, start_time: MediaTime, duration: MediaTime) -> BaseTimelineElement {
    BaseTimelineElement {
        id: id.to_string(),
        name: id.to_string(),
        start_time,
        duration,
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: None,
        animations: None,
        params: visual_params(),
    }
}

const IMAGE_DURATION: fn() -> MediaTime = || seconds(5.0);
const VIDEO_DURATION: fn() -> MediaTime = || seconds(10.0);
const TAIL_DURATION: fn() -> MediaTime = || seconds(5.0);
const PUSH_DELTA: fn() -> MediaTime = || seconds(3.0);

fn build_image_element() -> Element {
    Element::Image(ImageElement {
        base: base_element("image-1", MediaTime::ZERO, IMAGE_DURATION()),
        media_id: "media-image-1".to_string(),
        hidden: None,
        effects: None,
        masks: None,
    })
}

fn build_video_element(id: &str, start_time: MediaTime, duration: MediaTime) -> Element {
    Element::Video(VideoElement {
        base: BaseTimelineElement {
            source_duration: Some(duration),
            ..base_element(id, start_time, duration)
        },
        media_id: format!("media-{id}"),
        is_source_audio_enabled: None,
        hidden: None,
        retime: None,
        effects: None,
        masks: None,
    })
}

fn seed_tracks() -> (SceneTracks, String) {
    let main = Track::Video {
        id: "main-track".to_string(),
        name: "Main Track".to_string(),
        elements: vec![
            build_image_element(),
            build_video_element("video-1", IMAGE_DURATION(), VIDEO_DURATION()),
            build_video_element("video-2", IMAGE_DURATION() + VIDEO_DURATION(), TAIL_DURATION()),
        ],
        muted: false,
        hidden: false,
    };
    (
        SceneTracks {
            overlay: Vec::new(),
            main,
            audio: Vec::new(),
        },
        "main-track".to_string(),
    )
}

fn get_element<'a>(tracks: &'a SceneTracks, element_id: &str) -> Option<&'a Element> {
    tracks
        .main
        .elements()
        .iter()
        .find(|element| element.base().id == element_id)
}

#[test]
fn resize_and_push_land_in_one_applied_update() {
    let (tracks, track_id) = seed_tracks();

    let members = build_resize_members(
        &tracks,
        &[ElementRef {
            track_id: track_id.clone(),
            element_id: "image-1".to_string(),
        }],
    );
    let result = compute_group_resize(&members, ResizeSide::Right, PUSH_DELTA(), FPS);

    let mut updated = tracks.clone();
    for update in &result.updates {
        updated = update_element_in_scene_tracks(
            &updated,
            &update.track_id,
            &update.element_id,
            |element| {
                let mut next = element.clone();
                let base = next.base_mut();
                base.trim_start = update.patch.trim_start;
                base.trim_end = update.patch.trim_end;
                base.start_time = update.patch.start_time;
                base.duration = update.patch.duration;
                next
            },
            None,
        );
    }

    assert_eq!(
        get_element(&updated, "image-1").unwrap().base().duration,
        IMAGE_DURATION() + PUSH_DELTA()
    );
    assert_eq!(
        get_element(&updated, "video-1").unwrap().base().start_time,
        IMAGE_DURATION() + PUSH_DELTA()
    );
    assert_eq!(
        get_element(&updated, "video-2").unwrap().base().start_time,
        IMAGE_DURATION() + VIDEO_DURATION() + PUSH_DELTA()
    );
}
