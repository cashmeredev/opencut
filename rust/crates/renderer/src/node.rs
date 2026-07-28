use std::path::PathBuf;

use compositor::BlendMode;
use scene::{Effect, ElementAnimations, Mask, ParamValues, RetimeConfig, TextElement};
use time::MediaTime;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    pub scale_x: f64,
    pub scale_y: f64,
    pub position_x: f64,
    pub position_y: f64,
    pub rotate: f64,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Audio,
}

#[derive(Clone, Debug)]
pub struct MediaSource {
    pub id: String,
    pub media_type: MediaType,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct VisualParams {
    pub duration: MediaTime,
    pub time_offset: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    pub retime: Option<RetimeConfig>,
    pub transform: Transform,
    pub animations: Option<ElementAnimations>,
    pub opacity: f64,
    pub blend_mode: Option<BlendMode>,
    pub effects: Vec<Effect>,
    pub masks: Vec<Mask>,
}

#[derive(Clone, Debug)]
pub struct TextNodeParams {
    pub element: TextElement,
    pub transform: Transform,
    pub opacity: f64,
    pub blend_mode: Option<BlendMode>,
    pub canvas_center_x: f64,
    pub canvas_center_y: f64,
    pub canvas_height: f64,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug)]
pub struct GraphicNodeParams {
    pub visual: VisualParams,
    pub definition_id: String,
    pub params: ParamValues,
}

#[derive(Clone, Debug)]
pub struct BlurBackgroundParams {
    pub media_id: String,
    pub path: PathBuf,
    pub media_type: MediaType,
    pub duration: MediaTime,
    pub time_offset: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    pub retime: Option<RetimeConfig>,
    pub blur_intensity: f64,
}

#[derive(Clone, Debug)]
pub struct EffectLayerParams {
    pub effect_type: String,
    pub effect_params: ParamValues,
    pub time_offset: MediaTime,
    pub duration: MediaTime,
}

#[derive(Clone, Debug)]
pub struct StickerNodeParams {
    pub visual: VisualParams,
    pub sticker_id: String,
    pub intrinsic_width: Option<f64>,
    pub intrinsic_height: Option<f64>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Root { duration: MediaTime, children: Vec<Node> },
    Color { color: String },
    BlurBackground(BlurBackgroundParams),
    EffectLayer(EffectLayerParams),
    Video { visual: VisualParams, media_id: String, path: PathBuf },
    Image { visual: VisualParams, path: PathBuf, max_source_size: Option<u32> },
    Sticker(StickerNodeParams),
    Graphic(GraphicNodeParams),
    Text(TextNodeParams),
}

impl Node {
    pub fn children(&self) -> &[Node] {
        match self {
            Node::Root { children, .. } => children,
            _ => &[],
        }
    }
}
