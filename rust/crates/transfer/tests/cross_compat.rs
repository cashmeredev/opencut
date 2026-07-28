use transfer::*;

#[test]
fn parses_web_built_archive() {
    let archive = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/web-built.ocp"))
        .expect("web-built fixture");
    let parsed = parse_project_archive(&archive).expect("parse web archive");

    assert_eq!(
        parsed
            .project
            .get("nested")
            .and_then(|n| n.get("value"))
            .and_then(|v| v.as_u64()),
        Some(42)
    );
    assert_eq!(parsed.media.len(), 1);
    assert_eq!(parsed.media[0].entry, "media/web-asset-web_clip.mov");
    assert_eq!(parsed.media[0].metadata.media_type, "video");
    assert_eq!(parsed.media[0].metadata.width, Some(1280.0));
    assert_eq!(
        parsed.read_media_data("media/web-asset-web_clip.mov").unwrap(),
        &[9, 8, 7, 6]
    );
}

#[test]
fn writes_rust_built_archive_for_web_parse() {
    let archive = build_project_archive(
        r#"{"version":7,"metadata":{"id":"rust-built"}}"#,
        vec![ArchiveMediaInput {
            metadata: ArchivedMediaMetadata {
                id: "rust-asset".into(),
                name: "rust audio.mp3".into(),
                media_type: "audio".into(),
                file_type: "audio/mpeg".into(),
                last_modified: 1_753_000_000_000.0,
                width: None,
                height: None,
                duration: Some(1.5),
                ephemeral: None,
                thumbnail_url: None,
            },
            data: vec![4, 3, 2, 1],
        }],
    )
    .expect("build");
    let parsed = parse_project_archive(&archive).expect("re-parse");
    assert_eq!(parsed.media[0].entry, "media/rust-asset-rust_audio.mp3");
    assert_eq!(
        parsed
            .read_media_data("media/rust-asset-rust_audio.mp3")
            .unwrap(),
        &[4, 3, 2, 1]
    );
}
