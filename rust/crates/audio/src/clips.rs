use std::path::PathBuf;

use animation::{has_keyframes_for_path, resolve_number_at_time};
use scene::{
    AudioElement, BaseTimelineElement, Element, ElementAnimations, ParamValue, RetimeConfig,
    SceneTracks, Track,
};
use time::MediaTime;

pub const MIN_RETIME_RATE: f64 = 0.01;
pub const MAX_RETIME_RATE: f64 = 5.0;
pub const VOLUME_DB_MIN: f64 = -60.0;
pub const VOLUME_DB_MAX: f64 = 20.0;

pub fn clamp_retime_rate(rate: f64) -> f64 {
    if !rate.is_finite() || rate <= 0.0 {
        return 1.0;
    }
    rate.clamp(MIN_RETIME_RATE, MAX_RETIME_RATE)
}

pub fn source_time_at_clip_time(clip_time: f64, retime: Option<&RetimeConfig>) -> f64 {
    clip_time * clamp_retime_rate(retime.map(|config| config.rate).unwrap_or(1.0))
}

pub fn clamp_db(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(VOLUME_DB_MIN, VOLUME_DB_MAX)
}

pub fn db_to_linear(db: f64) -> f64 {
    10_f64.powf(clamp_db(db) / 20.0)
}

#[derive(Debug, Clone)]
pub struct AudibleClip {
    pub element_id: String,
    pub path: PathBuf,
    pub start_time: MediaTime,
    pub duration: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    pub volume_db: f64,
    pub retime: Option<RetimeConfig>,
    pub animations: Option<ElementAnimations>,
}

impl AudibleClip {
    pub fn end_time(&self) -> MediaTime {
        self.start_time + self.duration
    }

    pub fn effective_rate(&self) -> f64 {
        clamp_retime_rate(self.retime.map(|config| config.rate).unwrap_or(1.0))
    }

    pub fn requires_maintain_pitch(&self) -> bool {
        self.retime.is_some_and(|config| {
            config.maintain_pitch == Some(true)
                && config.rate.is_finite()
                && config.rate > 0.0
                && clamp_retime_rate(config.rate) != 1.0
        })
    }

    pub fn has_animated_volume(&self) -> bool {
        has_keyframes_for_path(self.animations.as_ref(), "volume")
    }

    pub fn gain_at(&self, local_time: MediaTime) -> f64 {
        db_to_linear(resolve_number_at_time(
            self.volume_db,
            self.animations.as_ref(),
            "volume",
            local_time,
        ))
    }

    pub fn static_gain(&self) -> f64 {
        db_to_linear(self.volume_db)
    }
}

fn track_audio_muted(track: &Track) -> bool {
    match track {
        Track::Video { muted, .. } | Track::Audio { muted, .. } => *muted,
        _ => false,
    }
}

fn element_source(element: &Element) -> Option<(&BaseTimelineElement, &str, Option<RetimeConfig>)> {
    match element {
        Element::Audio(AudioElement::Upload(upload)) => {
            Some((&upload.base, upload.media_id.as_str(), upload.retime))
        }
        Element::Audio(AudioElement::Library(library)) => {
            Some((&library.base, library.source_url.as_str(), library.retime))
        }
        Element::Video(video) => {
            if video.hidden == Some(true) || video.is_source_audio_enabled == Some(false) {
                return None;
            }
            Some((&video.base, video.media_id.as_str(), video.retime))
        }
        _ => None,
    }
}

fn collect_from_track(
    track: &Track,
    media: &dyn Fn(&str) -> Option<PathBuf>,
    range: (MediaTime, MediaTime),
    clips: &mut Vec<AudibleClip>,
) {
    if track_audio_muted(track) {
        return;
    }
    for element in track.elements() {
        let Some((base, source_key, retime)) = element_source(element) else {
            continue;
        };
        if base.duration <= MediaTime::ZERO {
            continue;
        }
        let end_time = base.start_time + base.duration;
        if end_time <= range.0 || base.start_time >= range.1 {
            continue;
        }
        if base.params.get("muted") == Some(&ParamValue::Bool(true)) {
            continue;
        }
        let volume_db = match base.params.get("volume") {
            Some(ParamValue::Number(value)) => *value,
            _ => 0.0,
        };
        let Some(path) = media(source_key) else {
            continue;
        };
        clips.push(AudibleClip {
            element_id: base.id.clone(),
            path,
            start_time: base.start_time,
            duration: base.duration,
            trim_start: base.trim_start,
            trim_end: base.trim_end,
            volume_db,
            retime,
            animations: base.animations.clone(),
        });
    }
}

pub fn collect_audible_elements(
    tracks: &SceneTracks,
    media: &dyn Fn(&str) -> Option<PathBuf>,
    range: (MediaTime, MediaTime),
) -> Vec<AudibleClip> {
    let mut clips = Vec::new();
    for track in &tracks.overlay {
        collect_from_track(track, media, range, &mut clips);
    }
    collect_from_track(&tracks.main, media, range, &mut clips);
    for track in &tracks.audio {
        collect_from_track(track, media, range, &mut clips);
    }
    clips
}
