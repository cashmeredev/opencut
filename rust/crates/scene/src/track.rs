use serde::{Deserialize, Serialize};

use crate::element::Element;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Track {
    Video {
        id: String,
        name: String,
        elements: Vec<Element>,
        muted: bool,
        hidden: bool,
    },
    Text {
        id: String,
        name: String,
        elements: Vec<Element>,
        hidden: bool,
    },
    Audio {
        id: String,
        name: String,
        elements: Vec<Element>,
        muted: bool,
    },
    Graphic {
        id: String,
        name: String,
        elements: Vec<Element>,
        hidden: bool,
    },
    Effect {
        id: String,
        name: String,
        elements: Vec<Element>,
        hidden: bool,
    },
}

impl Track {
    pub fn id(&self) -> &str {
        match self {
            Track::Video { id, .. }
            | Track::Text { id, .. }
            | Track::Audio { id, .. }
            | Track::Graphic { id, .. }
            | Track::Effect { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Track::Video { name, .. }
            | Track::Text { name, .. }
            | Track::Audio { name, .. }
            | Track::Graphic { name, .. }
            | Track::Effect { name, .. } => name,
        }
    }

    pub fn elements(&self) -> &[Element] {
        match self {
            Track::Video { elements, .. }
            | Track::Text { elements, .. }
            | Track::Audio { elements, .. }
            | Track::Graphic { elements, .. }
            | Track::Effect { elements, .. } => elements,
        }
    }

    pub fn elements_mut(&mut self) -> &mut Vec<Element> {
        match self {
            Track::Video { elements, .. }
            | Track::Text { elements, .. }
            | Track::Audio { elements, .. }
            | Track::Graphic { elements, .. }
            | Track::Effect { elements, .. } => elements,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SceneTracks {
    pub overlay: Vec<Track>,
    pub main: Track,
    pub audio: Vec<Track>,
}
