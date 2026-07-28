use scene::{
    BaseTimelineElement, Element, ParamValue, ParamValues, RetimeConfig, SceneTracks, Track,
    VideoElement,
};
use time::MediaTime;
use timeline::update_pipeline::{ElementPatch, ElementUpdateContext, apply_element_update};

fn build_video_element() -> Element {
    Element::Video(VideoElement {
        base: BaseTimelineElement {
            id: "video-1".to_string(),
            name: "Video 1".to_string(),
            start_time: MediaTime::ZERO,
            duration: MediaTime::from_ticks(10),
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
        media_id: "media-1".to_string(),
        is_source_audio_enabled: None,
        hidden: None,
        retime: None,
        effects: None,
        masks: None,
    })
}

fn build_tracks(element: Element) -> SceneTracks {
    SceneTracks {
        overlay: Vec::new(),
        main: Track::Video {
            id: "main-track".to_string(),
            name: "Main".to_string(),
            elements: vec![element],
            muted: false,
            hidden: false,
        },
        audio: Vec::new(),
    }
}

#[test]
fn rounds_retimed_durations_back_to_integer_media_time() {
    let element = build_video_element();
    let tracks = build_tracks(element.clone());

    let updated = apply_element_update(
        &element,
        &ElementPatch {
            retime: Some(Some(RetimeConfig {
                rate: 1.5,
                maintain_pitch: None,
            })),
            ..ElementPatch::default()
        },
        &ElementUpdateContext {
            tracks: &tracks,
            track_id: tracks.main.id(),
        },
    );

    assert_eq!(updated.base().duration, MediaTime::from_ticks(7));
    assert_eq!(timeline::element_retime(&updated).unwrap().rate, 1.5);
}

#[test]
fn start_time_on_main_track_snaps_to_zero_before_the_earliest_element() {
    let earliest = {
        let mut element = build_video_element();
        element.base_mut().id = "video-2".to_string();
        element.base_mut().start_time = MediaTime::from_ticks(100);
        element
    };
    let element = build_video_element();
    let tracks = build_tracks(earliest);
    let tracks = SceneTracks {
        main: Track::Video {
            id: "main-track".to_string(),
            name: "Main".to_string(),
            elements: vec![tracks.main.elements()[0].clone(), element.clone()],
            muted: false,
            hidden: false,
        },
        ..tracks
    };

    let updated = apply_element_update(
        &element,
        &ElementPatch {
            start_time: Some(MediaTime::from_ticks(50)),
            ..ElementPatch::default()
        },
        &ElementUpdateContext {
            tracks: &tracks,
            track_id: "main-track",
        },
    );

    assert_eq!(updated.base().start_time, MediaTime::ZERO);
}

#[test]
fn start_time_on_overlay_track_only_clamps_negative_values() {
    let element = build_video_element();
    let tracks = SceneTracks {
        overlay: vec![Track::Video {
            id: "overlay-track".to_string(),
            name: "Overlay".to_string(),
            elements: vec![element.clone()],
            muted: false,
            hidden: false,
        }],
        main: Track::Video {
            id: "main-track".to_string(),
            name: "Main".to_string(),
            elements: Vec::new(),
            muted: false,
            hidden: false,
        },
        audio: Vec::new(),
    };

    let updated = apply_element_update(
        &element,
        &ElementPatch {
            start_time: Some(MediaTime::from_ticks(-20)),
            ..ElementPatch::default()
        },
        &ElementUpdateContext {
            tracks: &tracks,
            track_id: "overlay-track",
        },
    );

    assert_eq!(updated.base().start_time, MediaTime::ZERO);
}
