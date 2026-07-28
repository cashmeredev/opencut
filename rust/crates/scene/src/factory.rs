use chrono::{DateTime, TimeZone, Utc};
use time::{FrameRate, MediaTime};
use uuid::Uuid;

use crate::project::{
    Background, CanvasSize, CanvasSizeMode, Project, ProjectMetadata, ProjectSettings,
};
use crate::scene::Scene;
use crate::track::{SceneTracks, Track};

pub const MAIN_TRACK_NAME: &str = "Main Track";
pub const CURRENT_PROJECT_VERSION: u32 = 32;
pub const DEFAULT_BACKGROUND_COLOR: &str = "#000000";
pub const DEFAULT_CANVAS_SIZE: CanvasSize = CanvasSize {
    width: 1920,
    height: 1080,
};
pub const DEFAULT_FPS: FrameRate = FrameRate {
    numerator: 30,
    denominator: 1,
};

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

fn now_millis() -> DateTime<Utc> {
    Utc.timestamp_millis_opt(Utc::now().timestamp_millis()).unwrap()
}

pub fn build_default_scene(name: impl Into<String>, is_main: bool) -> Scene {
    let now = now_millis();
    Scene {
        id: generate_id(),
        name: name.into(),
        is_main,
        tracks: SceneTracks {
            overlay: Vec::new(),
            main: Track::Video {
                id: generate_id(),
                name: MAIN_TRACK_NAME.to_string(),
                elements: Vec::new(),
                muted: false,
                hidden: false,
            },
            audio: Vec::new(),
        },
        bookmarks: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}

pub fn new_project(name: impl Into<String>) -> Project {
    let main_scene = build_default_scene("Main scene", true);
    let now = now_millis();
    Project {
        metadata: ProjectMetadata {
            id: generate_id(),
            name: name.into(),
            thumbnail: None,
            duration: MediaTime::ZERO,
            created_at: now,
            updated_at: now,
        },
        current_scene_id: main_scene.id.clone(),
        scenes: vec![main_scene],
        settings: ProjectSettings {
            fps: DEFAULT_FPS,
            canvas_size: DEFAULT_CANVAS_SIZE,
            canvas_size_mode: Some(CanvasSizeMode::Preset),
            last_custom_canvas_size: None,
            original_canvas_size: None,
            background: Background::Color {
                color: DEFAULT_BACKGROUND_COLOR.to_string(),
            },
        },
        version: CURRENT_PROJECT_VERSION,
        timeline_view_state: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_matches_web_defaults() {
        let project = new_project("Untitled Project");
        assert_eq!(project.version, 32);
        assert_eq!(project.scenes.len(), 1);
        assert!(project.scenes[0].is_main);
        assert_eq!(project.current_scene_id, project.scenes[0].id);
        assert_eq!(project.settings.canvas_size.width, 1920);
        assert_eq!(project.settings.fps.numerator, 30);
        match &project.scenes[0].tracks.main {
            Track::Video { name, .. } => assert_eq!(name, "Main Track"),
            _ => panic!("main track must be video"),
        }
        match &project.settings.background {
            Background::Color { color } => assert_eq!(color, "#000000"),
            _ => panic!("default background must be color"),
        }
    }

    #[test]
    fn new_project_round_trips_through_json() {
        let project = new_project("Test");
        let json = serde_json::to_value(&project).unwrap();
        let parsed: Project = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, project);
    }
}
