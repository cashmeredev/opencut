#![allow(dead_code)]

use scene::{
    Background, BaseTimelineElement, CanvasSize, Element, ImageElement, ParamValue,
    ParamValues, Project, ProjectMetadata, ProjectSettings, Scene, VideoElement,
};
use time::{FrameRate, MediaTime};

pub const SPLIT_SECONDS: f64 = 4.0;
pub const FREEZE_SECONDS: f64 = 3.0;
pub const VIDEO_SECONDS: f64 = 10.0;

pub fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).expect("finite seconds")
}

pub fn default_params() -> ParamValues {
    ParamValues::from([
        ("transform.positionX".to_string(), ParamValue::Number(0.0)),
        ("transform.positionY".to_string(), ParamValue::Number(0.0)),
        ("transform.scaleX".to_string(), ParamValue::Number(1.0)),
        ("transform.scaleY".to_string(), ParamValue::Number(1.0)),
        ("transform.rotate".to_string(), ParamValue::Number(0.0)),
        ("opacity".to_string(), ParamValue::Number(1.0)),
    ])
}

pub fn base_element(id: &str, name: &str, duration: MediaTime) -> BaseTimelineElement {
    BaseTimelineElement {
        id: id.to_string(),
        name: name.to_string(),
        duration,
        start_time: MediaTime::ZERO,
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: None,
        animations: None,
        params: default_params(),
    }
}

pub fn build_video_element() -> Element {
    Element::Video(VideoElement {
        base: base_element("video-1", "video-1", seconds(VIDEO_SECONDS)),
        media_id: "media-video-1".to_string(),
        is_source_audio_enabled: None,
        hidden: None,
        retime: None,
        effects: None,
        masks: None,
    })
}

pub fn build_freeze_image_element() -> Element {
    let mut base = base_element("freeze-template", "Freeze frame", seconds(FREEZE_SECONDS));
    base.start_time = seconds(SPLIT_SECONDS);
    Element::Image(ImageElement {
        base,
        media_id: "media-freeze-1".to_string(),
        hidden: None,
        effects: None,
        masks: None,
    })
}

pub fn active_scene(project: &Project) -> &Scene {
    project
        .scenes
        .iter()
        .find(|scene| scene.id == project.current_scene_id)
        .expect("active scene")
}

pub fn seed_project_with_video() -> (Project, String) {
    let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .expect("valid date")
        .to_utc();

    let mut scene: Scene = timeline::scenes::build_default_scene(
        "scene-1".to_string(),
        "main-track-1".to_string(),
        "Main scene".to_string(),
        true,
        now,
    );
    scene.tracks.main.elements_mut().push(build_video_element());
    let track_id = scene.tracks.main.id().to_string();

    let project = Project {
        metadata: ProjectMetadata {
            id: "project-1".to_string(),
            name: "Test project".to_string(),
            thumbnail: None,
            duration: seconds(VIDEO_SECONDS),
            created_at: now,
            updated_at: now,
        },
        scenes: vec![scene],
        current_scene_id: "scene-1".to_string(),
        settings: ProjectSettings {
            fps: FrameRate::new(30, 1),
            canvas_size: CanvasSize {
                width: 1920,
                height: 1080,
            },
            canvas_size_mode: None,
            last_custom_canvas_size: None,
            original_canvas_size: None,
            background: Background::Color {
                color: "#000000".to_string(),
            },
        },
        version: 1,
        timeline_view_state: None,
    };

    (project, track_id)
}
