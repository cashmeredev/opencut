use scene::{Element, Track};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TrackType {
    Video,
    Text,
    Audio,
    Graphic,
    Effect,
}

impl TrackType {
    pub fn as_str(self) -> &'static str {
        match self {
            TrackType::Video => "video",
            TrackType::Text => "text",
            TrackType::Audio => "audio",
            TrackType::Graphic => "graphic",
            TrackType::Effect => "effect",
        }
    }

    pub fn of_track(track: &Track) -> TrackType {
        match track {
            Track::Video { .. } => TrackType::Video,
            Track::Text { .. } => TrackType::Text,
            Track::Audio { .. } => TrackType::Audio,
            Track::Graphic { .. } => TrackType::Graphic,
            Track::Effect { .. } => TrackType::Effect,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ElementType {
    Audio,
    Video,
    Image,
    Text,
    Sticker,
    Graphic,
    Effect,
}

impl ElementType {
    pub fn as_str(self) -> &'static str {
        match self {
            ElementType::Audio => "audio",
            ElementType::Video => "video",
            ElementType::Image => "image",
            ElementType::Text => "text",
            ElementType::Sticker => "sticker",
            ElementType::Graphic => "graphic",
            ElementType::Effect => "effect",
        }
    }

    pub fn of_element(element: &Element) -> ElementType {
        match element {
            Element::Audio(_) => ElementType::Audio,
            Element::Video(_) => ElementType::Video,
            Element::Image(_) => ElementType::Image,
            Element::Text(_) => ElementType::Text,
            Element::Sticker(_) => ElementType::Sticker,
            Element::Graphic(_) => ElementType::Graphic,
            Element::Effect(_) => ElementType::Effect,
        }
    }
}
