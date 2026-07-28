use scene::{
    BaseTimelineElement, Element, ElementRef, ParamValue, ParamValues, RetimeConfig, SceneTracks,
    Track, VideoElement,
};
use time::MediaTime;
use timeline::split::{RetainSide, split_elements};

fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).unwrap()
}

fn build_video_element(id: &str, duration: MediaTime, retime: Option<RetimeConfig>) -> Element {
    Element::Video(VideoElement {
        base: BaseTimelineElement {
            id: id.to_string(),
            name: id.to_string(),
            start_time: MediaTime::ZERO,
            duration,
            trim_start: MediaTime::ZERO,
            trim_end: MediaTime::ZERO,
            source_duration: None,
            animations: None,
            params: ParamValues::from([
                ("transform.positionX".to_string(), ParamValue::Number(0.0)),
                ("transform.positionY".to_string(), ParamValue::Number(0.0)),
                ("transform.scaleX".to_string(), ParamValue::Number(1.0)),
                ("transform.scaleY".to_string(), ParamValue::Number(1.0)),
                ("transform.rotate".to_string(), ParamValue::Number(0.0)),
                ("opacity".to_string(), ParamValue::Number(1.0)),
            ]),
        },
        media_id: format!("media-{id}"),
        is_source_audio_enabled: None,
        hidden: None,
        retime,
        effects: None,
        masks: None,
    })
}

fn seed_tracks(element: Element) -> (SceneTracks, ElementRef) {
    let reference = ElementRef {
        track_id: "main-track".to_string(),
        element_id: element.base().id.clone(),
    };
    (
        SceneTracks {
            overlay: Vec::new(),
            main: Track::Video {
                id: "main-track".to_string(),
                name: "Main Track".to_string(),
                elements: vec![element],
                muted: false,
                hidden: false,
            },
            audio: Vec::new(),
        },
        reference,
    )
}

#[test]
fn split_produces_left_and_right_halves_with_shifted_trims() {
    let (tracks, reference) = seed_tracks(build_video_element("video-1", seconds(10.0), None));

    let result = split_elements(&tracks, &[reference], seconds(4.0), RetainSide::Both);

    assert_eq!(result.right_side_elements.len(), 1);
    let elements = result.tracks.main.elements();
    assert_eq!(elements.len(), 2);

    let left = &elements[0];
    assert_eq!(left.base().id, "video-1");
    assert_eq!(left.base().name, "video-1 (left)");
    assert_eq!(left.base().start_time, MediaTime::ZERO);
    assert_eq!(left.base().duration, seconds(4.0));
    assert_eq!(left.base().trim_start, MediaTime::ZERO);
    assert_eq!(left.base().trim_end, seconds(6.0));

    let right = &elements[1];
    assert_eq!(right.base().id, result.right_side_elements[0].element_id);
    assert_eq!(right.base().name, "video-1 (right)");
    assert_eq!(right.base().start_time, seconds(4.0));
    assert_eq!(right.base().duration, seconds(6.0));
    assert_eq!(right.base().trim_start, seconds(4.0));
    assert_eq!(right.base().trim_end, MediaTime::ZERO);
}

#[test]
fn split_scales_source_spans_by_the_retime_rate() {
    let retime = RetimeConfig {
        rate: 2.0,
        maintain_pitch: None,
    };
    let (tracks, reference) =
        seed_tracks(build_video_element("video-1", seconds(10.0), Some(retime)));

    let result = split_elements(&tracks, &[reference], seconds(4.0), RetainSide::Both);

    let elements = result.tracks.main.elements();
    let left = &elements[0];
    let right = &elements[1];
    assert_eq!(left.base().trim_end, seconds(12.0));
    assert_eq!(right.base().trim_start, seconds(8.0));
    assert_eq!(
        left.base().trim_start + left.base().duration * 2 + left.base().trim_end,
        right.base().trim_start + right.base().duration * 2 + right.base().trim_end
    );
    assert_eq!(timeline::element_retime(left), Some(retime));
    assert_eq!(timeline::element_retime(right), Some(retime));
}

#[test]
fn retain_left_keeps_only_the_left_half() {
    let (tracks, reference) = seed_tracks(build_video_element("video-1", seconds(10.0), None));

    let result = split_elements(&tracks, &[reference], seconds(4.0), RetainSide::Left);

    assert_eq!(result.right_side_elements.len(), 0);
    let elements = result.tracks.main.elements();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].base().id, "video-1");
    assert_eq!(elements[0].base().duration, seconds(4.0));
    assert_eq!(elements[0].base().trim_end, seconds(6.0));
}

#[test]
fn retain_right_keeps_only_the_right_half_with_a_new_id() {
    let (tracks, reference) = seed_tracks(build_video_element("video-1", seconds(10.0), None));

    let result = split_elements(&tracks, &[reference], seconds(4.0), RetainSide::Right);

    assert_eq!(result.right_side_elements.len(), 1);
    let elements = result.tracks.main.elements();
    assert_eq!(elements.len(), 1);
    let right = &elements[0];
    assert_ne!(right.base().id, "video-1");
    assert_eq!(right.base().id, result.right_side_elements[0].element_id);
    assert_eq!(right.base().start_time, seconds(4.0));
    assert_eq!(right.base().duration, seconds(6.0));
    assert_eq!(right.base().trim_start, seconds(4.0));
}

#[test]
fn split_outside_the_element_bounds_is_a_no_op() {
    let (tracks, reference) = seed_tracks(build_video_element("video-1", seconds(10.0), None));

    for split_time in [MediaTime::ZERO, seconds(10.0), seconds(12.0)] {
        let result = split_elements(
            &tracks,
            std::slice::from_ref(&reference),
            split_time,
            RetainSide::Both,
        );
        assert_eq!(result.right_side_elements.len(), 0);
        assert_eq!(result.tracks.main.elements().len(), 1);
        assert_eq!(
            result.tracks.main.elements()[0].base().duration,
            seconds(10.0)
        );
    }
}

#[test]
fn shifting_the_right_side_and_inserting_into_the_gap_keeps_layout_consistent() {
    let (tracks, reference) = seed_tracks(build_video_element("video-1", seconds(10.0), None));

    let result = split_elements(&tracks, &[reference], seconds(4.0), RetainSide::Both);
    let right_ref = result.right_side_elements[0].clone();

    let shifted = timeline::update_element_in_scene_tracks(
        &result.tracks,
        &right_ref.track_id,
        &right_ref.element_id,
        |element| {
            let mut next = element.clone();
            next.base_mut().start_time = element.base().start_time + seconds(3.0);
            next
        },
        None,
    );

    let elements = shifted.main.elements();
    assert_eq!(elements[0].base().duration, seconds(4.0));
    assert_eq!(elements[1].base().start_time, seconds(7.0));
    assert_eq!(elements[1].base().duration, seconds(6.0));
}
