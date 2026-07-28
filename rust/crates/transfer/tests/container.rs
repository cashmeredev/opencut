use std::io::Write as _;

use transfer::*;

fn sample_media() -> Vec<ArchiveMediaInput> {
    vec![
        ArchiveMediaInput {
            metadata: ArchivedMediaMetadata {
                id: "asset-1".into(),
                name: "clip one.mp4".into(),
                media_type: "video".into(),
                file_type: "video/mp4".into(),
                last_modified: 1_753_000_000_000.0,
                width: Some(1920.0),
                height: Some(1080.0),
                duration: Some(12.5),
                ephemeral: None,
                thumbnail_url: None,
            },
            data: vec![1, 2, 3, 4],
        },
        ArchiveMediaInput {
            metadata: ArchivedMediaMetadata {
                id: "asset-2".into(),
                name: ".hidden".into(),
                media_type: "audio".into(),
                file_type: "audio/wav".into(),
                last_modified: 1_753_000_000_001.0,
                width: None,
                height: None,
                duration: None,
                ephemeral: Some(true),
                thumbnail_url: None,
            },
            data: vec![5, 6, 7],
        },
    ]
}

#[test]
fn sanitize_matches_web_rules() {
    assert_eq!(sanitize_file_name("clip one.mp4"), "clip_one.mp4");
    assert_eq!(sanitize_file_name(".."), "file");
    assert_eq!(sanitize_file_name("...weird..name.."), "weird..name..");
    assert_eq!(sanitize_file_name("a  b"), "a_b");
    assert_eq!(sanitize_file_name("   "), "file");
    assert_eq!(sanitize_file_name("caf\u{00e9}.mov"), "caf_.mov");
}

#[test]
fn archive_round_trip() {
    let project_json = r#"{"version":7,"metadata":{"id":"p1"}}"#;
    let archive = build_project_archive(project_json, sample_media()).expect("build");
    let parsed = parse_project_archive(&archive).expect("parse");

    assert_eq!(
        parsed.project.get("version").and_then(|v| v.as_u64()),
        Some(7)
    );
    assert_eq!(parsed.media.len(), 2);
    assert_eq!(parsed.media[0].entry, "media/asset-1-clip_one.mp4");
    assert_eq!(parsed.media[1].entry, "media/asset-2-hidden");
    assert_eq!(
        parsed.read_media_data("media/asset-1-clip_one.mp4").unwrap(),
        &[1, 2, 3, 4]
    );
    assert_eq!(parsed.read_media_data("media/asset-2-hidden").unwrap(), &[5, 6, 7]);
}

#[test]
fn archive_without_media_is_valid() {
    let archive = build_project_archive("{}", Vec::new()).expect("build");
    let parsed = parse_project_archive(&archive).expect("parse");
    assert!(parsed.media.is_empty());
}

#[test]
fn rejects_garbage() {
    let error = parse_project_archive(b"not a zip").unwrap_err();
    assert!(error.to_string().contains("unreadable zip file"));
}

#[test]
fn rejects_missing_project_json() {
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut cursor);
        let options: zip::write::SimpleFileOptions = Default::default();
        writer.start_file("other.txt", options).unwrap();
        writer.write_all(b"hi").unwrap();
        writer.finish().unwrap();
    }
    let error = parse_project_archive(&cursor.into_inner()).unwrap_err();
    assert!(error.to_string().contains("project.json is missing"));
}
