use chrono::{DateTime, Utc};
use scene::{Scene, SceneTracks};
use time::MediaTime;

use crate::placement::MAIN_TRACK_NAME;
use crate::placement::track_factory::build_empty_track;
use crate::types::TrackType;

pub fn get_main_scene(scenes: &[Scene]) -> Option<&Scene> {
    scenes.iter().find(|scene| scene.is_main)
}

pub fn ensure_main_scene(scenes: &[Scene], next_id: &mut dyn FnMut() -> String, now: DateTime<Utc>) -> Vec<Scene> {
    if scenes.iter().any(|scene| scene.is_main) {
        return scenes.to_vec();
    }

    let main_scene = build_default_scene(
        next_id(),
        next_id(),
        "Main scene".to_string(),
        true,
        now,
    );
    let mut next = Vec::with_capacity(scenes.len() + 1);
    next.push(main_scene);
    next.extend(scenes.iter().cloned());
    next
}

pub fn build_default_scene(
    id: impl Into<String>,
    main_track_id: impl Into<String>,
    name: String,
    is_main: bool,
    now: DateTime<Utc>,
) -> Scene {
    Scene {
        id: id.into(),
        name,
        is_main,
        tracks: SceneTracks {
            overlay: Vec::new(),
            main: build_empty_track(
                main_track_id,
                TrackType::Video,
                Some(MAIN_TRACK_NAME.to_string()),
            ),
            audio: Vec::new(),
        },
        bookmarks: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}

pub struct SceneDeleteCheck {
    pub can_delete: bool,
    pub reason: Option<String>,
}

pub fn can_delete_scene(scene: &Scene) -> SceneDeleteCheck {
    if scene.is_main {
        return SceneDeleteCheck {
            can_delete: false,
            reason: Some("Cannot delete main scene".to_string()),
        };
    }
    SceneDeleteCheck {
        can_delete: true,
        reason: None,
    }
}

pub fn get_fallback_scene_after_delete<'a>(
    scenes: &'a [Scene],
    deleted_scene_id: &str,
    current_scene_id: Option<&str>,
) -> Option<&'a Scene> {
    if current_scene_id != Some(deleted_scene_id) {
        return scenes.iter().find(|scene| Some(scene.id.as_str()) == current_scene_id);
    }
    get_main_scene(scenes)
}

pub fn find_current_scene<'a>(
    scenes: &'a [Scene],
    current_scene_id: &str,
) -> Option<&'a Scene> {
    scenes
        .iter()
        .find(|scene| scene.id == current_scene_id)
        .or_else(|| get_main_scene(scenes))
        .or_else(|| scenes.first())
}

pub fn get_project_duration_from_scenes(scenes: &[Scene]) -> MediaTime {
    let Some(main_scene) = get_main_scene(scenes).or_else(|| scenes.first()) else {
        return MediaTime::ZERO;
    };

    crate::calculate_total_duration(&main_scene.tracks)
}

pub fn update_scene_in_array(
    scenes: &[Scene],
    scene_id: &str,
    update: impl Fn(&Scene) -> Scene,
) -> Vec<Scene> {
    scenes
        .iter()
        .map(|scene| {
            if scene.id == scene_id {
                update(scene)
            } else {
                scene.clone()
            }
        })
        .collect()
}
