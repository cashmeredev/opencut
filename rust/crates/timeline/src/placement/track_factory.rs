use scene::Track;

use crate::types::TrackType;

pub const DEFAULT_VIDEO_TRACK_NAME: &str = "Video track";
pub const DEFAULT_TEXT_TRACK_NAME: &str = "Text track";
pub const DEFAULT_AUDIO_TRACK_NAME: &str = "Audio track";
pub const DEFAULT_GRAPHIC_TRACK_NAME: &str = "Graphic track";
pub const DEFAULT_EFFECT_TRACK_NAME: &str = "Effect track";

pub fn default_track_name(track_type: TrackType) -> &'static str {
    match track_type {
        TrackType::Video => DEFAULT_VIDEO_TRACK_NAME,
        TrackType::Text => DEFAULT_TEXT_TRACK_NAME,
        TrackType::Audio => DEFAULT_AUDIO_TRACK_NAME,
        TrackType::Graphic => DEFAULT_GRAPHIC_TRACK_NAME,
        TrackType::Effect => DEFAULT_EFFECT_TRACK_NAME,
    }
}

pub fn build_empty_track(
    id: impl Into<String>,
    track_type: TrackType,
    name: Option<String>,
) -> Track {
    let id = id.into();
    let name = name.unwrap_or_else(|| default_track_name(track_type).to_string());

    match track_type {
        TrackType::Video => Track::Video {
            id,
            name,
            elements: Vec::new(),
            muted: false,
            hidden: false,
        },
        TrackType::Text => Track::Text {
            id,
            name,
            elements: Vec::new(),
            hidden: false,
        },
        TrackType::Audio => Track::Audio {
            id,
            name,
            elements: Vec::new(),
            muted: false,
        },
        TrackType::Graphic => Track::Graphic {
            id,
            name,
            elements: Vec::new(),
            hidden: false,
        },
        TrackType::Effect => Track::Effect {
            id,
            name,
            elements: Vec::new(),
            hidden: false,
        },
    }
}
