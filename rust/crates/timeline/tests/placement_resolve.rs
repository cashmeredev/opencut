use scene::{
    AudioElement, BaseTimelineElement, Element, GraphicElement, ParamValue, ParamValues,
    SceneTracks, TextElement, Track, UploadAudioElement, VideoElement,
};
use time::MediaTime;
use timeline::placement::{
    InsertDirection, PlacementResult, PlacementStrategy, PlacementSubject, PlacementTimeSpan,
    ResolveTrackPlacementParams, VerticalDragDirection, resolve_track_placement,
};
use timeline::types::{ElementType, TrackType};

fn ticks(value: i64) -> MediaTime {
    MediaTime::from_ticks(value)
}

fn base_element(id: &str, start_time: i64, duration: i64) -> BaseTimelineElement {
    BaseTimelineElement {
        id: id.to_string(),
        name: id.to_string(),
        start_time: ticks(start_time),
        duration: ticks(duration),
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: None,
        animations: None,
        params: ParamValues::new(),
    }
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

fn build_element(element_type: ElementType, id: &str, start_time: i64, duration: i64) -> Element {
    match element_type {
        ElementType::Audio => Element::Audio(AudioElement::Upload(UploadAudioElement {
            base: BaseTimelineElement {
                params: ParamValues::from([
                    ("volume".to_string(), ParamValue::Number(1.0)),
                    ("muted".to_string(), ParamValue::Bool(false)),
                ]),
                ..base_element(id, start_time, duration)
            },
            retime: None,
            media_id: format!("media-{id}"),
        })),
        ElementType::Graphic => Element::Graphic(GraphicElement {
            base: BaseTimelineElement {
                params: visual_params(),
                ..base_element(id, start_time, duration)
            },
            definition_id: format!("graphic-{id}"),
            hidden: None,
            effects: None,
            masks: None,
        }),
        ElementType::Text => Element::Text(TextElement {
            base: BaseTimelineElement {
                params: visual_params(),
                ..base_element(id, start_time, duration)
            },
            hidden: None,
            effects: None,
        }),
        ElementType::Video => Element::Video(VideoElement {
            base: BaseTimelineElement {
                params: visual_params(),
                ..base_element(id, start_time, duration)
            },
            media_id: format!("media-{id}"),
            is_source_audio_enabled: None,
            hidden: None,
            retime: None,
            effects: None,
            masks: None,
        }),
        _ => panic!("unsupported test element type"),
    }
}

fn build_track(track_type: TrackType, id: &str, elements: Vec<Element>) -> Track {
    match track_type {
        TrackType::Audio => Track::Audio {
            id: id.to_string(),
            name: id.to_string(),
            elements,
            muted: false,
        },
        TrackType::Graphic => Track::Graphic {
            id: id.to_string(),
            name: id.to_string(),
            elements,
            hidden: false,
        },
        TrackType::Text => Track::Text {
            id: id.to_string(),
            name: id.to_string(),
            elements,
            hidden: false,
        },
        TrackType::Video => Track::Video {
            id: id.to_string(),
            name: id.to_string(),
            elements,
            muted: false,
            hidden: false,
        },
        TrackType::Effect => Track::Effect {
            id: id.to_string(),
            name: id.to_string(),
            elements,
            hidden: false,
        },
    }
}

fn build_time_span(start_time: i64, duration: i64, exclude_element_id: Option<&str>) -> PlacementTimeSpan {
    PlacementTimeSpan {
        start_time: ticks(start_time),
        duration: ticks(duration),
        exclude_element_id: exclude_element_id.map(str::to_string),
    }
}

fn build_scene_tracks(overlay: Vec<Track>, main: Option<Track>, audio: Vec<Track>) -> SceneTracks {
    SceneTracks {
        overlay,
        main: main.unwrap_or_else(|| build_track(TrackType::Video, "video-main", Vec::new())),
        audio,
    }
}

fn resolve(
    tracks: SceneTracks,
    element_type: ElementType,
    time_spans: Vec<PlacementTimeSpan>,
    strategy: PlacementStrategy,
) -> Option<PlacementResult> {
    resolve_track_placement(&ResolveTrackPlacementParams {
        tracks,
        subject: PlacementSubject::ElementType(element_type),
        time_spans,
        strategy,
    })
}

#[test]
fn explicit_returns_the_requested_compatible_track() {
    let tracks = build_scene_tracks(
        vec![build_track(TrackType::Text, "text-1", Vec::new())],
        None,
        Vec::new(),
    );

    let result = resolve(
        tracks,
        ElementType::Text,
        vec![build_time_span(2, 3, None)],
        PlacementStrategy::Explicit {
            track_id: "text-1".to_string(),
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::ExistingTrack {
            track_id: "text-1".to_string(),
            track_index: 0,
            track_type: TrackType::Text,
            adjusted_start_time: None,
        })
    );
}

#[test]
fn explicit_rejects_missing_and_incompatible_tracks() {
    let tracks = || {
        build_scene_tracks(
            Vec::new(),
            Some(build_track(TrackType::Video, "video-1", Vec::new())),
            Vec::new(),
        )
    };

    let missing = resolve(
        tracks(),
        ElementType::Text,
        vec![build_time_span(0, 1, None)],
        PlacementStrategy::Explicit {
            track_id: "missing".to_string(),
        },
    );
    assert_eq!(missing, None);

    let incompatible = resolve(
        tracks(),
        ElementType::Text,
        vec![build_time_span(0, 1, None)],
        PlacementStrategy::Explicit {
            track_id: "video-1".to_string(),
        },
    );
    assert_eq!(incompatible, None);
}

#[test]
fn first_available_picks_the_first_compatible_track_without_overlap() {
    let tracks = build_scene_tracks(
        vec![
            build_track(
                TrackType::Text,
                "text-1",
                vec![build_element(ElementType::Text, "a", 0, 5)],
            ),
            build_track(TrackType::Text, "text-2", Vec::new()),
        ],
        None,
        Vec::new(),
    );

    let result = resolve(
        tracks,
        ElementType::Text,
        vec![build_time_span(2, 1, None)],
        PlacementStrategy::FirstAvailable,
    );

    assert_eq!(
        result,
        Some(PlacementResult::ExistingTrack {
            track_id: "text-2".to_string(),
            track_index: 1,
            track_type: TrackType::Text,
            adjusted_start_time: None,
        })
    );
}

#[test]
fn first_available_creates_a_new_track_when_all_compatible_tracks_are_full() {
    let tracks = build_scene_tracks(
        vec![build_track(
            TrackType::Graphic,
            "graphic-1",
            vec![build_element(ElementType::Graphic, "a", 0, 5)],
        )],
        Some(build_track(TrackType::Video, "video-main", Vec::new())),
        vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
    );

    let result = resolve(
        tracks,
        ElementType::Graphic,
        vec![build_time_span(1, 1, None)],
        PlacementStrategy::FirstAvailable,
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Graphic,
            insert_index: 0,
            insert_position: None,
        })
    );
}

#[test]
fn prefer_index_uses_the_preferred_track_when_it_fits() {
    let tracks = build_scene_tracks(
        Vec::new(),
        None,
        vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
    );

    let result = resolve(
        tracks,
        ElementType::Audio,
        vec![build_time_span(3, 2, None)],
        PlacementStrategy::PreferIndex {
            track_index: 0,
            hover_direction: InsertDirection::Below,
            vertical_drag_direction: None,
            create_new_track_only: false,
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 1,
            insert_position: Some(InsertDirection::Below),
        })
    );
}

#[test]
fn prefer_index_creates_a_new_overlay_track_above_the_main_track() {
    let tracks = build_scene_tracks(
        Vec::new(),
        Some(build_track(TrackType::Video, "video-main", Vec::new())),
        vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
    );

    let result = resolve(
        tracks,
        ElementType::Graphic,
        vec![build_time_span(1, 2, None)],
        PlacementStrategy::PreferIndex {
            track_index: 1,
            hover_direction: InsertDirection::Below,
            vertical_drag_direction: None,
            create_new_track_only: false,
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Graphic,
            insert_index: 0,
            insert_position: Some(InsertDirection::Above),
        })
    );
}

#[test]
fn prefer_index_keeps_audio_tracks_below_the_main_track() {
    let tracks = build_scene_tracks(
        vec![build_track(TrackType::Text, "text-1", Vec::new())],
        Some(build_track(TrackType::Video, "video-main", Vec::new())),
        vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
    );

    let result = resolve(
        tracks,
        ElementType::Audio,
        vec![build_time_span(0, 1, None)],
        PlacementStrategy::PreferIndex {
            track_index: 0,
            hover_direction: InsertDirection::Above,
            vertical_drag_direction: None,
            create_new_track_only: true,
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 2,
            insert_position: Some(InsertDirection::Below),
        })
    );
}

#[test]
fn above_source_tries_the_track_above_source_then_any_compatible_track() {
    let tracks = build_scene_tracks(
        vec![
            build_track(TrackType::Text, "text-top", Vec::new()),
            build_track(
                TrackType::Text,
                "text-middle",
                vec![build_element(ElementType::Text, "a", 0, 5)],
            ),
            build_track(TrackType::Text, "text-source", Vec::new()),
        ],
        None,
        Vec::new(),
    );

    let result = resolve(
        tracks,
        ElementType::Text,
        vec![build_time_span(1, 1, None)],
        PlacementStrategy::AboveSource {
            source_track_index: 2,
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::ExistingTrack {
            track_id: "text-top".to_string(),
            track_index: 0,
            track_type: TrackType::Text,
            adjusted_start_time: None,
        })
    );
}

#[test]
fn above_source_creates_a_new_overlay_track_in_the_overlay_zone_when_none_fit() {
    let tracks = build_scene_tracks(
        vec![
            build_track(
                TrackType::Text,
                "text-top",
                vec![build_element(ElementType::Text, "a", 0, 5)],
            ),
            build_track(
                TrackType::Text,
                "text-source",
                vec![build_element(ElementType::Text, "b", 0, 5)],
            ),
        ],
        None,
        Vec::new(),
    );

    let result = resolve(
        tracks,
        ElementType::Text,
        vec![build_time_span(1, 1, None)],
        PlacementStrategy::AboveSource {
            source_track_index: 1,
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Text,
            insert_index: 0,
            insert_position: None,
        })
    );
}

#[test]
fn always_new_honors_highest_and_default_insertion_rules() {
    use timeline::placement::NewTrackPosition;

    let tracks = || {
        build_scene_tracks(
            Vec::new(),
            Some(build_track(TrackType::Video, "video-main", Vec::new())),
            vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
        )
    };

    let highest = resolve(
        tracks(),
        ElementType::Audio,
        Vec::new(),
        PlacementStrategy::AlwaysNew {
            position: NewTrackPosition::Highest,
        },
    );
    assert_eq!(
        highest,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 1,
            insert_position: None,
        })
    );

    let default = resolve(
        tracks(),
        ElementType::Audio,
        Vec::new(),
        PlacementStrategy::AlwaysNew {
            position: NewTrackPosition::Default,
        },
    );
    assert_eq!(
        default,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 2,
            insert_position: None,
        })
    );
}

#[test]
fn batch_time_spans_reject_tracks_when_any_span_overlaps() {
    let tracks = build_scene_tracks(
        Vec::new(),
        None,
        vec![build_track(
            TrackType::Audio,
            "audio-1",
            vec![
                build_element(ElementType::Audio, "a", 0, 2),
                build_element(ElementType::Audio, "b", 5, 2),
            ],
        )],
    );

    let result = resolve(
        tracks,
        ElementType::Audio,
        vec![build_time_span(3, 1, None), build_time_span(6, 1, None)],
        PlacementStrategy::FirstAvailable,
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 1,
            insert_position: None,
        })
    );
}

#[test]
fn handles_main_only_timelines_and_track_type_derivation() {
    use timeline::placement::NewTrackPosition;

    let main_only = resolve(
        build_scene_tracks(Vec::new(), None, Vec::new()),
        ElementType::Video,
        vec![build_time_span(0, 3, None)],
        PlacementStrategy::PreferIndex {
            track_index: 0,
            hover_direction: InsertDirection::Below,
            vertical_drag_direction: None,
            create_new_track_only: true,
        },
    );
    assert_eq!(
        main_only,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Video,
            insert_index: 0,
            insert_position: Some(InsertDirection::Above),
        })
    );

    let audio_default = resolve(
        build_scene_tracks(
            Vec::new(),
            None,
            vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
        ),
        ElementType::Audio,
        Vec::new(),
        PlacementStrategy::AlwaysNew {
            position: NewTrackPosition::Default,
        },
    );
    assert_eq!(
        audio_default,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 2,
            insert_position: None,
        })
    );
}

#[test]
fn existing_track_on_main_video_includes_adjusted_start_time_when_start_snaps() {
    let tracks = build_scene_tracks(
        Vec::new(),
        Some(build_track(
            TrackType::Video,
            "video-main",
            vec![build_element(ElementType::Video, "a", 5, 5)],
        )),
        Vec::new(),
    );

    let result = resolve(
        tracks,
        ElementType::Video,
        vec![build_time_span(2, 2, None)],
        PlacementStrategy::Explicit {
            track_id: "video-main".to_string(),
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::ExistingTrack {
            track_id: "video-main".to_string(),
            track_index: 0,
            track_type: TrackType::Video,
            adjusted_start_time: Some(MediaTime::ZERO),
        })
    );
}

#[test]
fn prefer_index_uses_vertical_drag_direction_when_hovered_track_is_incompatible() {
    let tracks = build_scene_tracks(
        vec![build_track(TrackType::Text, "text-1", Vec::new())],
        Some(build_track(TrackType::Video, "video-main", Vec::new())),
        vec![build_track(TrackType::Audio, "audio-1", Vec::new())],
    );

    let result = resolve(
        tracks,
        ElementType::Audio,
        vec![build_time_span(0, 1, None)],
        PlacementStrategy::PreferIndex {
            track_index: 0,
            hover_direction: InsertDirection::Above,
            vertical_drag_direction: Some(VerticalDragDirection::Down),
            create_new_track_only: false,
        },
    );

    assert_eq!(
        result,
        Some(PlacementResult::NewTrack {
            track_type: TrackType::Audio,
            insert_index: 2,
            insert_position: Some(InsertDirection::Below),
        })
    );
}
