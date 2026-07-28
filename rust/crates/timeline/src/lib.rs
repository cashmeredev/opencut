use scene::{AudioElement, Element, RetimeConfig, SceneTracks};
use time::MediaTime;

pub mod audio_separation;
pub mod creation;
pub mod defaults;
pub mod drag_utils;
pub mod placement;
pub mod resize;
pub mod retime;
pub mod ripple;
pub mod scenes;
pub mod snap;
pub mod split;
pub mod track_capabilities;
pub mod track_element_update;
pub mod types;
pub mod update_pipeline;

pub use audio_separation::*;
pub use creation::*;
pub use track_capabilities::*;
pub use track_element_update::*;
pub use types::{ElementType, TrackType};

pub fn calculate_total_duration(tracks: &SceneTracks) -> MediaTime {
    placement::ordered_tracks(tracks)
        .iter()
        .flat_map(|track| track.elements().iter())
        .map(|element| element.base().start_time + element.base().duration)
        .fold(MediaTime::ZERO, MediaTime::max)
}

pub fn is_visual_element(element: &Element) -> bool {
    matches!(
        element,
        Element::Video(_)
            | Element::Image(_)
            | Element::Text(_)
            | Element::Sticker(_)
            | Element::Graphic(_)
    )
}

pub fn is_maskable_element(element: &Element) -> bool {
    matches!(element, Element::Video(_) | Element::Image(_) | Element::Graphic(_))
}

pub fn is_retimable_element(element: &Element) -> bool {
    matches!(element, Element::Video(_) | Element::Audio(_))
}

pub fn can_element_have_audio(element: &Element) -> bool {
    matches!(element, Element::Audio(_) | Element::Video(_))
}

pub fn can_element_be_hidden(element: &Element) -> bool {
    is_visual_element(element)
}

pub fn has_element_effects(element: &Element) -> bool {
    let effects = match element {
        Element::Video(video) => video.effects.as_ref(),
        Element::Image(image) => image.effects.as_ref(),
        Element::Text(text) => text.effects.as_ref(),
        Element::Sticker(sticker) => sticker.effects.as_ref(),
        Element::Graphic(graphic) => graphic.effects.as_ref(),
        _ => None,
    };
    effects.is_some_and(|effects| !effects.is_empty())
}

pub fn has_media_id(element: &Element) -> bool {
    matches!(
        element,
        Element::Audio(AudioElement::Upload(_)) | Element::Video(_) | Element::Image(_)
    )
}

pub fn element_retime(element: &Element) -> Option<RetimeConfig> {
    match element {
        Element::Video(video) => video.retime,
        Element::Audio(AudioElement::Upload(audio)) => audio.retime,
        Element::Audio(AudioElement::Library(audio)) => audio.retime,
        _ => None,
    }
}

pub fn set_element_retime(element: &mut Element, retime: Option<RetimeConfig>) {
    match element {
        Element::Video(video) => video.retime = retime,
        Element::Audio(AudioElement::Upload(audio)) => audio.retime = retime,
        Element::Audio(AudioElement::Library(audio)) => audio.retime = retime,
        _ => {}
    }
}
