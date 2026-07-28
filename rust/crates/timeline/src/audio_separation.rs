use scene::{Element, ElementAnimations, ParamValue, ParamValues, UploadAudioElement, VideoElement};

use crate::defaults::DEFAULT_VOLUME;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaAudioState {
    pub has_audio: Option<bool>,
}

pub fn is_source_audio_enabled(element: &VideoElement) -> bool {
    element.is_source_audio_enabled != Some(false)
}

pub fn is_source_audio_separated(element: &VideoElement) -> bool {
    !is_source_audio_enabled(element)
}

pub fn can_extract_source_audio(element: &Element, media_asset: Option<&MediaAudioState>) -> bool {
    let Element::Video(video) = element else {
        return false;
    };
    is_source_audio_enabled(video) && media_asset.is_some_and(|media| media.has_audio != Some(false))
}

pub fn can_recover_source_audio(element: &Element) -> bool {
    let Element::Video(video) = element else {
        return false;
    };
    is_source_audio_separated(video)
}

pub fn can_toggle_source_audio(element: &Element, media_asset: Option<&MediaAudioState>) -> bool {
    can_recover_source_audio(element) || can_extract_source_audio(element, media_asset)
}

pub fn does_element_have_enabled_audio(
    element: &Element,
    media_asset: Option<&MediaAudioState>,
) -> bool {
    match element {
        Element::Audio(_) => true,
        Element::Video(video) => {
            media_asset.is_some_and(|media| media.has_audio != Some(false))
                && is_source_audio_enabled(video)
        }
        _ => false,
    }
}

pub fn build_separated_audio_element(
    source_element: &VideoElement,
    id: impl Into<String>,
) -> UploadAudioElement {
    let source_base = &source_element.base;

    let mut params = ParamValues::new();
    let volume = match source_base.params.get("volume") {
        Some(ParamValue::Number(volume)) => *volume,
        _ => DEFAULT_VOLUME,
    };
    params.insert("volume".to_string(), ParamValue::Number(volume));
    params.insert(
        "muted".to_string(),
        ParamValue::Bool(source_base.params.get("muted") == Some(&ParamValue::Bool(true))),
    );

    UploadAudioElement {
        base: scene::BaseTimelineElement {
            id: id.into(),
            name: source_base.name.clone(),
            duration: source_base.duration,
            start_time: source_base.start_time,
            trim_start: source_base.trim_start,
            trim_end: source_base.trim_end,
            source_duration: source_base.source_duration,
            animations: clone_volume_animations(source_base.animations.as_ref()),
            params,
        },
        retime: source_element.retime,
        media_id: source_element.media_id.clone(),
    }
}

pub fn get_source_audio_action_label(element: &VideoElement) -> &'static str {
    if is_source_audio_separated(element) {
        "Recover audio"
    } else {
        "Extract audio"
    }
}

fn clone_volume_animations(animations: Option<&ElementAnimations>) -> Option<ElementAnimations> {
    let volume_data = animations?.get("volume")?;
    let mut volume_only = ElementAnimations::new();
    volume_only.insert("volume".to_string(), volume_data.clone());
    animation::clone_animations(Some(&volume_only), true)
}
