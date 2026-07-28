use crate::types::{ElementType, TrackType};

pub fn get_track_type_for_element_type(element_type: ElementType) -> TrackType {
    match element_type {
        ElementType::Audio => TrackType::Audio,
        ElementType::Text => TrackType::Text,
        ElementType::Sticker | ElementType::Graphic => TrackType::Graphic,
        ElementType::Effect => TrackType::Effect,
        ElementType::Video | ElementType::Image => TrackType::Video,
    }
}

pub fn can_element_go_on_track(element_type: ElementType, track_type: TrackType) -> bool {
    get_track_type_for_element_type(element_type) == track_type
}

pub struct TrackCompatibility {
    pub is_valid: bool,
    pub error_message: Option<String>,
}

pub fn validate_element_track_compatibility(
    element_type: ElementType,
    track_type: TrackType,
) -> TrackCompatibility {
    let is_valid = can_element_go_on_track(element_type, track_type);

    if !is_valid {
        return TrackCompatibility {
            is_valid: false,
            error_message: Some(format!(
                "{} elements cannot be placed on {} tracks",
                element_type.as_str(),
                track_type.as_str(),
            )),
        };
    }

    TrackCompatibility {
        is_valid: true,
        error_message: None,
    }
}
