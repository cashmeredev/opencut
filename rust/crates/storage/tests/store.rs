use chrono::{DateTime, Utc};
use scene::{
    Background, BaseTimelineElement, CanvasSize, Element, Project, ProjectMetadata,
    ProjectSettings, Scene, SceneTracks, Track, VideoElement,
};
use storage::ProjectStore;
use tempfile::TempDir;
use time::{FrameRate, MediaTime};
use transfer::ArchivedMediaMetadata;

fn date(day: u8) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&format!("2026-01-{day:02}T00:00:00Z"))
        .unwrap()
        .with_timezone(&Utc)
}

fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).unwrap()
}

fn sample_project(id: &str) -> Project {
    let scene_id = format!("{id}-scene");
    let element = Element::Video(VideoElement {
        base: BaseTimelineElement {
            id: "element-1".to_string(),
            name: "Clip".to_string(),
            duration: seconds(5.0),
            start_time: MediaTime::ZERO,
            trim_start: MediaTime::ZERO,
            trim_end: MediaTime::ZERO,
            source_duration: Some(seconds(10.0)),
            animations: None,
            params: Default::default(),
        },
        media_id: "media-1".to_string(),
        is_source_audio_enabled: None,
        hidden: None,
        retime: None,
        effects: None,
        masks: None,
    });
    let scene = Scene {
        id: scene_id.clone(),
        name: "Main scene".to_string(),
        is_main: true,
        tracks: SceneTracks {
            overlay: vec![],
            main: Track::Video {
                id: "track-1".to_string(),
                name: "Main".to_string(),
                elements: vec![element],
                muted: false,
                hidden: false,
            },
            audio: vec![],
        },
        bookmarks: vec![],
        created_at: date(1),
        updated_at: date(1),
    };
    Project {
        metadata: ProjectMetadata {
            id: id.to_string(),
            name: "Sample Project".to_string(),
            thumbnail: None,
            duration: seconds(5.0),
            created_at: date(1),
            updated_at: date(2),
        },
        scenes: vec![scene],
        current_scene_id: scene_id,
        settings: ProjectSettings {
            fps: FrameRate::FPS_30,
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
        version: 14,
        timeline_view_state: None,
    }
}

fn sample_media(id: &str, name: &str) -> ArchivedMediaMetadata {
    ArchivedMediaMetadata {
        id: id.to_string(),
        name: name.to_string(),
        media_type: "video".to_string(),
        file_type: "video/mp4".to_string(),
        last_modified: 1_700_000_000_000.0,
        width: Some(1920.0),
        height: Some(1080.0),
        duration: Some(5.0),
        ephemeral: None,
        thumbnail_url: None,
    }
}

fn store() -> (TempDir, ProjectStore) {
    let dir = TempDir::new().unwrap();
    let store = ProjectStore::new(dir.path().join("projects"));
    (dir, store)
}

#[test]
fn project_round_trip() {
    let (_dir, store) = store();
    let project = sample_project("project-1");

    store.create_project(&project).unwrap();
    assert!(store.create_project(&project).is_err());

    let mut updated = project.clone();
    updated.metadata.name = "Renamed Project".to_string();
    store.save_project(&updated).unwrap();

    let listed = store.list_projects().unwrap();
    assert_eq!(listed, vec![updated.metadata.clone()]);

    let loaded = store.load_project("project-1").unwrap();
    assert_eq!(loaded, updated);

    let raw = std::fs::read_to_string(
        store.project_dir("project-1").join("project.json"),
    )
    .unwrap();
    assert!(raw.contains("\"currentSceneId\""));
    assert!(raw.contains("\"createdAt\""));
}

#[test]
fn list_projects_sorts_by_updated_descending() {
    let (_dir, store) = store();
    store.create_project(&sample_project("project-old")).unwrap();
    let mut newer = sample_project("project-new");
    newer.metadata.updated_at = date(20);
    store.create_project(&newer).unwrap();

    let listed = store.list_projects().unwrap();
    let ids: Vec<&str> = listed.iter().map(|meta| meta.id.as_str()).collect();
    assert_eq!(ids, vec!["project-new", "project-old"]);
}

#[test]
fn media_round_trip() {
    let (_dir, store) = store();
    let project = sample_project("project-1");
    store.create_project(&project).unwrap();

    let metadata = sample_media("media-1", "my clip.mp4");
    let bytes = b"fake video bytes";
    store.add_media("project-1", &metadata, bytes).unwrap();

    let listed = store.list_media("project-1").unwrap();
    assert_eq!(listed, vec![metadata.clone()]);

    let path = store.media_path("project-1", "media-1").unwrap();
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        "media-1-my_clip.mp4"
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);

    let manifest = std::fs::read_to_string(
        store
            .project_dir("project-1")
            .join("media")
            .join("manifest.json"),
    )
    .unwrap();
    let parsed: Vec<ArchivedMediaMetadata> = serde_json::from_str(&manifest).unwrap();
    assert_eq!(parsed, vec![metadata.clone()]);
    assert!(!parsed[0].media_type.is_empty());

    store.delete_media("project-1", "media-1").unwrap();
    assert!(store.list_media("project-1").unwrap().is_empty());
    assert!(store.media_path("project-1", "media-1").is_err());
    assert!(!path.exists());
}

#[test]
fn add_media_replaces_existing_id() {
    let (_dir, store) = store();
    let project = sample_project("project-1");
    store.create_project(&project).unwrap();

    let first = sample_media("media-1", "first.mp4");
    store.add_media("project-1", &first, b"first").unwrap();
    let second = sample_media("media-1", "second.mp4");
    store.add_media("project-1", &second, b"second").unwrap();

    let listed = store.list_media("project-1").unwrap();
    assert_eq!(listed, vec![second]);
    let path = store.media_path("project-1", "media-1").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second");
    let media_dir = store.project_dir("project-1").join("media");
    let files: Vec<_> = std::fs::read_dir(&media_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(files.len(), 2);
}

#[test]
fn export_import_round_trip() {
    let (_dir_a, source) = store();
    let project = sample_project("project-1");
    source.create_project(&project).unwrap();
    let metadata = sample_media("media-1", "my clip.mp4");
    let bytes = b"fake video bytes";
    source.add_media("project-1", &metadata, bytes).unwrap();

    let archive = source.export_ocp("project-1").unwrap();

    let (_dir_b, target) = store();
    let imported_id = target.import_ocp(&archive).unwrap();
    assert_eq!(imported_id, "project-1");

    let loaded = target.load_project(&imported_id).unwrap();
    assert_eq!(loaded, project);

    let listed = target.list_media(&imported_id).unwrap();
    assert_eq!(listed, vec![metadata]);
    let path = target.media_path(&imported_id, "media-1").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn delete_project_removes_directory() {
    let (_dir, store) = store();
    let project = sample_project("project-1");
    store.create_project(&project).unwrap();
    store
        .add_media("project-1", &sample_media("media-1", "clip.mp4"), b"data")
        .unwrap();

    let project_dir = store.project_dir("project-1");
    assert!(project_dir.is_dir());

    store.delete_project("project-1").unwrap();
    assert!(!project_dir.exists());
    assert!(store.load_project("project-1").is_err());
    assert!(store.list_projects().unwrap().is_empty());
    assert!(store.delete_project("project-1").is_err());
}

#[test]
fn import_id_collision_gets_new_id() {
    let (_dir_a, source) = store();
    let project = sample_project("project-1");
    source.create_project(&project).unwrap();
    let archive = source.export_ocp("project-1").unwrap();

    let (_dir_b, target) = store();
    let first = target.import_ocp(&archive).unwrap();
    let second = target.import_ocp(&archive).unwrap();

    assert_eq!(first, "project-1");
    assert_ne!(first, second);
    assert_eq!(second, "project-1-1");

    let first_project = target.load_project(&first).unwrap();
    let second_project = target.load_project(&second).unwrap();
    assert_eq!(first_project.metadata.id, first);
    assert_eq!(second_project.metadata.id, second);
    assert_eq!(second_project.scenes, first_project.scenes);

    let listed = target.list_projects().unwrap();
    assert_eq!(listed.len(), 2);
}
