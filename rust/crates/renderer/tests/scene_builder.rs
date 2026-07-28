use std::path::PathBuf;

use renderer::node::{MediaSource, MediaType, Node};
use renderer::{BuildSceneParams, build_scene};
use scene::{
    Background, BaseTimelineElement, CanvasSize, Element, ImageElement, ParamValues, SceneTracks,
    TextElement, Track, VideoElement,
};
use time::MediaTime;

fn base(id: &str, start: i64) -> BaseTimelineElement {
    BaseTimelineElement {
        id: id.to_string(),
        name: id.to_string(),
        duration: MediaTime::from_ticks(1000),
        start_time: MediaTime::from_ticks(start),
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: None,
        animations: None,
        params: ParamValues::new(),
    }
}

fn video_element(id: &str, start: i64, media_id: &str) -> Element {
    Element::Video(VideoElement {
        base: base(id, start),
        media_id: media_id.to_string(),
        is_source_audio_enabled: None,
        hidden: None,
        retime: None,
        effects: None,
        masks: None,
    })
}

fn image_element(id: &str, start: i64, media_id: &str) -> Element {
    Element::Image(ImageElement {
        base: base(id, start),
        media_id: media_id.to_string(),
        hidden: None,
        effects: None,
        masks: None,
    })
}

fn text_element(id: &str, start: i64, hidden: Option<bool>) -> Element {
    Element::Text(TextElement {
        base: base(id, start),
        hidden,
        effects: None,
    })
}

fn video_track(id: &str, hidden: bool, elements: Vec<Element>) -> Track {
    Track::Video {
        id: id.to_string(),
        name: id.to_string(),
        elements,
        muted: false,
        hidden,
    }
}

fn media(id: &str, media_type: MediaType) -> MediaSource {
    MediaSource {
        id: id.to_string(),
        media_type,
        path: PathBuf::from(format!("/media/{id}")),
    }
}

fn build(tracks: &SceneTracks, media: &[MediaSource], background: &Background) -> Vec<Node> {
    let params = BuildSceneParams {
        canvas_size: CanvasSize {
            width: 1920,
            height: 1080,
        },
        tracks,
        media,
        duration: MediaTime::from_ticks(10_000),
        background,
        is_preview: false,
    };
    match build_scene(&params) {
        Node::Root { children, .. } => children,
        _ => panic!("expected root"),
    }
}

fn transparent() -> Background {
    Background::Color {
        color: "transparent".into(),
    }
}

#[test]
fn skips_elements_without_media_and_mismatched_types() {
    let tracks = SceneTracks {
        overlay: vec![],
        main: video_track(
            "main",
            false,
            vec![
                video_element("v1", 0, "m1"),
                video_element("v2", 0, "missing"),
                image_element("i1", 0, "m1"),
            ],
        ),
        audio: vec![],
    };
    let assets = [media("m1", MediaType::Video)];
    let children = build(&tracks, &assets, &transparent());

    assert_eq!(children.len(), 1);
    assert!(matches!(&children[0], Node::Video { media_id, .. } if media_id == "m1"));
}

#[test]
fn main_track_renders_below_overlay() {
    let tracks = SceneTracks {
        overlay: vec![video_track("overlay", false, vec![text_element("t1", 0, None)])],
        main: video_track("main", false, vec![video_element("v1", 0, "m1")]),
        audio: vec![],
    };
    let assets = [media("m1", MediaType::Video)];
    let children = build(&tracks, &assets, &transparent());

    assert_eq!(children.len(), 2);
    assert!(matches!(&children[0], Node::Video { .. }));
    assert!(matches!(&children[1], Node::Text(_)));
}

#[test]
fn hidden_elements_and_tracks_are_skipped() {
    let tracks = SceneTracks {
        overlay: vec![video_track("hidden-track", true, vec![text_element("t2", 0, None)])],
        main: video_track(
            "main",
            false,
            vec![video_element("v1", 0, "m1"), text_element("t1", 0, Some(true))],
        ),
        audio: vec![],
    };
    let assets = [media("m1", MediaType::Video)];
    let children = build(&tracks, &assets, &transparent());

    assert_eq!(children.len(), 1);
    assert!(matches!(&children[0], Node::Video { .. }));
}

#[test]
fn elements_sort_by_start_time_then_id() {
    let tracks = SceneTracks {
        overlay: vec![],
        main: video_track(
            "main",
            false,
            vec![
                video_element("b", 100, "m1"),
                video_element("a", 100, "m1"),
                video_element("z", 50, "m1"),
            ],
        ),
        audio: vec![],
    };
    let assets = [media("m1", MediaType::Video)];
    let children = build(&tracks, &assets, &transparent());

    let starts: Vec<i64> = children
        .iter()
        .map(|node| match node {
            Node::Video { visual, .. } => visual.time_offset.as_ticks(),
            _ => panic!("expected video"),
        })
        .collect();
    assert_eq!(starts, [50, 100, 100]);
}

#[test]
fn color_background_comes_first_unless_transparent() {
    let tracks = SceneTracks {
        overlay: vec![],
        main: video_track("main", false, vec![video_element("v1", 0, "m1")]),
        audio: vec![],
    };
    let assets = [media("m1", MediaType::Video)];

    let with_color = build(
        &tracks,
        &assets,
        &Background::Color {
            color: "#112233".into(),
        },
    );
    assert!(matches!(&with_color[0], Node::Color { color } if color == "#112233"));
    assert_eq!(with_color.len(), 2);

    let transparent_children = build(&tracks, &assets, &transparent());
    assert_eq!(transparent_children.len(), 1);
}

#[test]
fn blur_background_emits_backdrop_nodes_for_main_media() {
    let tracks = SceneTracks {
        overlay: vec![],
        main: video_track("main", false, vec![video_element("v1", 0, "m1")]),
        audio: vec![],
    };
    let assets = [media("m1", MediaType::Video)];
    let children = build(
        &tracks,
        &assets,
        &Background::Blur {
            blur_intensity: 12.0,
        },
    );

    assert_eq!(children.len(), 2);
    match &children[0] {
        Node::BlurBackground(params) => {
            assert_eq!(params.blur_intensity, 12.0);
            assert_eq!(params.media_id, "m1");
        }
        _ => panic!("expected blur background first"),
    }
    assert!(matches!(&children[1], Node::Video { .. }));
}
