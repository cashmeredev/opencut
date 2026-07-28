use std::collections::BTreeMap;
use std::path::PathBuf;

use audio::collect_audible_elements;
use scene::{
    AudioElement, BaseTimelineElement, Element, LibraryAudioElement, ParamValue, RetimeConfig,
    SceneTracks, Track, UploadAudioElement, VideoElement,
};
use time::MediaTime;

fn seconds(value: f64) -> MediaTime {
    MediaTime::from_seconds_f64(value).expect("finite seconds")
}

fn base(id: &str, start: f64, duration: f64) -> BaseTimelineElement {
    BaseTimelineElement {
        id: id.to_string(),
        name: id.to_string(),
        duration: seconds(duration),
        start_time: seconds(start),
        trim_start: MediaTime::ZERO,
        trim_end: MediaTime::ZERO,
        source_duration: None,
        animations: None,
        params: BTreeMap::new(),
    }
}

fn upload_audio(id: &str, start: f64, duration: f64) -> Element {
    Element::Audio(AudioElement::Upload(UploadAudioElement {
        base: base(id, start, duration),
        retime: None,
        media_id: format!("media-{id}"),
    }))
}

fn library_audio(id: &str, start: f64, duration: f64) -> Element {
    Element::Audio(AudioElement::Library(LibraryAudioElement {
        base: base(id, start, duration),
        retime: None,
        source_url: format!("https://cdn.test/{id}.mp3"),
    }))
}

fn video(id: &str, start: f64, duration: f64) -> Element {
    Element::Video(VideoElement {
        base: base(id, start, duration),
        media_id: format!("media-{id}"),
        is_source_audio_enabled: None,
        hidden: None,
        retime: None,
        effects: None,
        masks: None,
    })
}

fn main_track(elements: Vec<Element>) -> Track {
    Track::Video {
        id: "main".to_string(),
        name: "main".to_string(),
        elements,
        muted: false,
        hidden: false,
    }
}

fn audio_track(id: &str, muted: bool, elements: Vec<Element>) -> Track {
    Track::Audio {
        id: id.to_string(),
        name: id.to_string(),
        elements,
        muted,
    }
}

fn tracks(main: Vec<Element>, audio: Vec<Track>) -> SceneTracks {
    SceneTracks {
        overlay: vec![],
        main: main_track(main),
        audio,
    }
}

fn resolve_any(key: &str) -> Option<PathBuf> {
    Some(PathBuf::from(key))
}

fn full_range() -> (MediaTime, MediaTime) {
    (MediaTime::ZERO, seconds(60.0))
}

#[test]
fn collects_upload_audio_and_video_with_source_audio() {
    let scene = tracks(
        vec![video("v1", 0.0, 2.0)],
        vec![audio_track("a", false, vec![upload_audio("u1", 1.0, 1.0)])],
    );
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[0].element_id, "v1");
    assert_eq!(clips[1].element_id, "u1");
    assert_eq!(clips[0].path, PathBuf::from("media-v1"));
    assert_eq!(clips[1].start_time, seconds(1.0));
    assert_eq!(clips[1].duration, seconds(1.0));
}

#[test]
fn library_audio_resolves_by_source_url() {
    let scene = tracks(
        vec![],
        vec![audio_track("a", false, vec![library_audio("lib", 0.0, 1.0)])],
    );
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].path, PathBuf::from("https://cdn.test/lib.mp3"));
}

#[test]
fn muted_track_excludes_its_elements() {
    let scene = tracks(
        vec![],
        vec![
            audio_track("muted-track", true, vec![upload_audio("u1", 0.0, 1.0)]),
            audio_track("live-track", false, vec![upload_audio("u2", 0.0, 1.0)]),
        ],
    );
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].element_id, "u2");
}

#[test]
fn muted_element_is_excluded() {
    let mut element = match upload_audio("u1", 0.0, 1.0) {
        Element::Audio(AudioElement::Upload(mut upload)) => {
            upload
                .base
                .params
                .insert("muted".to_string(), ParamValue::Bool(true));
            Element::Audio(AudioElement::Upload(upload))
        }
        other => other,
    };
    let scene = tracks(vec![], vec![audio_track("a", false, vec![element])]);
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert!(clips.is_empty());

    element = match upload_audio("u2", 0.0, 1.0) {
        Element::Audio(AudioElement::Upload(mut upload)) => {
            upload
                .base
                .params
                .insert("muted".to_string(), ParamValue::Bool(false));
            Element::Audio(AudioElement::Upload(upload))
        }
        other => other,
    };
    let scene = tracks(vec![], vec![audio_track("a", false, vec![element])]);
    assert_eq!(collect_audible_elements(&scene, &resolve_any, full_range()).len(), 1);
}

#[test]
fn hidden_video_is_excluded() {
    let hidden = Element::Video(VideoElement {
        hidden: Some(true),
        ..match video("v1", 0.0, 2.0) {
            Element::Video(v) => v,
            _ => unreachable!(),
        }
    });
    let scene = tracks(vec![hidden], vec![]);
    assert!(collect_audible_elements(&scene, &resolve_any, full_range()).is_empty());
}

#[test]
fn video_with_source_audio_disabled_is_excluded() {
    let disabled = Element::Video(VideoElement {
        is_source_audio_enabled: Some(false),
        ..match video("v1", 0.0, 2.0) {
            Element::Video(v) => v,
            _ => unreachable!(),
        }
    });
    let enabled = Element::Video(VideoElement {
        is_source_audio_enabled: Some(true),
        ..match video("v2", 0.0, 2.0) {
            Element::Video(v) => v,
            _ => unreachable!(),
        }
    });
    let scene = tracks(vec![disabled, enabled], vec![]);
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].element_id, "v2");
}

#[test]
fn range_overlap_trims_collection() {
    let scene = tracks(
        vec![video("v1", 2.0, 2.0), video("v2", 10.0, 2.0)],
        vec![],
    );
    let clips = collect_audible_elements(&scene, &resolve_any, (seconds(3.0), seconds(4.5)));
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].element_id, "v1");

    let clips = collect_audible_elements(&scene, &resolve_any, (seconds(4.0), seconds(10.0)));
    assert!(clips.is_empty());

    let clips = collect_audible_elements(&scene, &resolve_any, (seconds(4.0), seconds(10.5)));
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].element_id, "v2");
}

#[test]
fn volume_param_is_decibels_with_zero_default() {
    let quiet = match upload_audio("u1", 0.0, 1.0) {
        Element::Audio(AudioElement::Upload(mut upload)) => {
            upload
                .base
                .params
                .insert("volume".to_string(), ParamValue::Number(-6.0));
            Element::Audio(AudioElement::Upload(upload))
        }
        other => other,
    };
    let plain = upload_audio("u2", 0.0, 1.0);
    let scene = tracks(vec![], vec![audio_track("a", false, vec![quiet, plain])]);
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert_eq!(clips[0].volume_db, -6.0);
    assert_eq!(clips[1].volume_db, 0.0);
    assert!((clips[0].static_gain() - 0.5012).abs() < 0.001);
    assert_eq!(clips[1].static_gain(), 1.0);
}

#[test]
fn retime_and_trim_are_carried() {
    let element = match upload_audio("u1", 0.0, 0.5) {
        Element::Audio(AudioElement::Upload(mut upload)) => {
            upload.base.trim_start = seconds(0.25);
            upload.base.trim_end = seconds(0.25);
            upload.retime = Some(RetimeConfig {
                rate: 2.0,
                maintain_pitch: None,
            });
            Element::Audio(AudioElement::Upload(upload))
        }
        other => other,
    };
    let scene = tracks(vec![], vec![audio_track("a", false, vec![element])]);
    let clips = collect_audible_elements(&scene, &resolve_any, full_range());
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].trim_start, seconds(0.25));
    assert_eq!(clips[0].effective_rate(), 2.0);
    assert!(!clips[0].requires_maintain_pitch());
}

#[test]
fn unresolvable_media_is_excluded() {
    let scene = tracks(vec![video("v1", 0.0, 1.0)], vec![]);
    let clips = collect_audible_elements(&scene, &|_| None, full_range());
    assert!(clips.is_empty());
}
