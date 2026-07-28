use serde::{Deserialize, Serialize};
use time::MediaTime;

use crate::animation::ElementAnimations;
use crate::effect::Effect;
use crate::mask::Mask;
use crate::params::ParamValues;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ElementRef {
    pub track_id: String,
    pub element_id: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetimeConfig {
    pub rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintain_pitch: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseTimelineElement {
    pub id: String,
    pub name: String,
    pub duration: MediaTime,
    pub start_time: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_duration: Option<MediaTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animations: Option<ElementAnimations>,
    #[serde(default)]
    pub params: ParamValues,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub media_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_source_audio_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retime: Option<RetimeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub masks: Option<Vec<Mask>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub media_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub masks: Option<Vec<Mask>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StickerElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub sticker_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intrinsic_width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intrinsic_height: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphicElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub definition_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub masks: Option<Vec<Mask>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EffectElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub effect_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UploadAudioElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retime: Option<RetimeConfig>,
    pub media_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryAudioElement {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retime: Option<RetimeConfig>,
    pub source_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "sourceType", rename_all = "camelCase")]
pub enum AudioElement {
    Upload(UploadAudioElement),
    Library(LibraryAudioElement),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Element {
    Audio(AudioElement),
    Video(VideoElement),
    Image(ImageElement),
    Text(TextElement),
    Sticker(StickerElement),
    Graphic(GraphicElement),
    Effect(EffectElement),
}

impl Element {
    pub fn base(&self) -> &BaseTimelineElement {
        match self {
            Element::Audio(AudioElement::Upload(e)) => &e.base,
            Element::Audio(AudioElement::Library(e)) => &e.base,
            Element::Video(e) => &e.base,
            Element::Image(e) => &e.base,
            Element::Text(e) => &e.base,
            Element::Sticker(e) => &e.base,
            Element::Graphic(e) => &e.base,
            Element::Effect(e) => &e.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseTimelineElement {
        match self {
            Element::Audio(AudioElement::Upload(e)) => &mut e.base,
            Element::Audio(AudioElement::Library(e)) => &mut e.base,
            Element::Video(e) => &mut e.base,
            Element::Image(e) => &mut e.base,
            Element::Text(e) => &mut e.base,
            Element::Sticker(e) => &mut e.base,
            Element::Graphic(e) => &mut e.base,
            Element::Effect(e) => &mut e.base,
        }
    }
}
