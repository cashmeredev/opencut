use std::collections::HashMap;

use compositor::BlendMode;
use scene::{Background, CanvasSize, Element, ParamValues, SceneTracks, Track};
use time::MediaTime;

use crate::node::{
    BlurBackgroundParams, EffectLayerParams, GraphicNodeParams, MediaSource, MediaType, Node,
    StickerNodeParams, TextNodeParams, Transform, VisualParams,
};

const PREVIEW_MAX_IMAGE_SIZE: u32 = 2048;
pub const DEFAULT_BACKGROUND_BLUR_INTENSITY: f64 = 8.0;

pub struct BuildSceneParams<'a> {
    pub canvas_size: CanvasSize,
    pub tracks: &'a SceneTracks,
    pub media: &'a [MediaSource],
    pub duration: MediaTime,
    pub background: &'a Background,
    pub is_preview: bool,
}

pub fn build_transform_from_params(params: &ParamValues) -> Transform {
    Transform {
        scale_x: read_number_param(params, "transform.scaleX", 1.0),
        scale_y: read_number_param(params, "transform.scaleY", 1.0),
        position_x: read_number_param(params, "transform.positionX", 0.0),
        position_y: read_number_param(params, "transform.positionY", 0.0),
        rotate: read_number_param(params, "transform.rotate", 0.0),
    }
}

pub fn read_opacity_from_params(params: &ParamValues) -> f64 {
    read_number_param(params, "opacity", 1.0)
}

pub fn read_blend_mode_from_params(params: &ParamValues) -> Option<BlendMode> {
    let scene::ParamValue::String(value) = params.get("blendMode")? else {
        return None;
    };
    serde_json::from_value(serde_json::Value::String(value.clone())).ok()
}

fn read_number_param(params: &ParamValues, key: &str, fallback: f64) -> f64 {
    match params.get(key) {
        Some(scene::ParamValue::Number(value)) => *value,
        _ => fallback,
    }
}

fn visible_sorted_elements(track: &Track) -> Vec<&Element> {
    let mut elements: Vec<&Element> = track
        .elements()
        .iter()
        .filter(|element| !element_hidden(element))
        .collect();
    elements.sort_by(|a, b| {
        a.base()
            .start_time
            .cmp(&b.base().start_time)
            .then_with(|| a.base().id.cmp(&b.base().id))
    });
    elements
}

fn element_hidden(element: &Element) -> bool {
    match element {
        Element::Video(e) => e.hidden.unwrap_or(false),
        Element::Image(e) => e.hidden.unwrap_or(false),
        Element::Text(e) => e.hidden.unwrap_or(false),
        Element::Sticker(e) => e.hidden.unwrap_or(false),
        Element::Graphic(e) => e.hidden.unwrap_or(false),
        _ => false,
    }
}

fn visual_params(element: &Element) -> VisualParams {
    let base = element.base();
    let (retime, effects, masks) = match element {
        Element::Video(e) => (
            e.retime,
            e.effects.clone().unwrap_or_default(),
            e.masks.clone().unwrap_or_default(),
        ),
        Element::Image(e) => (
            None,
            e.effects.clone().unwrap_or_default(),
            e.masks.clone().unwrap_or_default(),
        ),
        Element::Sticker(e) => (None, e.effects.clone().unwrap_or_default(), Vec::new()),
        Element::Graphic(e) => (
            None,
            e.effects.clone().unwrap_or_default(),
            e.masks.clone().unwrap_or_default(),
        ),
        _ => (None, Vec::new(), Vec::new()),
    };
    VisualParams {
        duration: base.duration,
        time_offset: base.start_time,
        trim_start: base.trim_start,
        trim_end: base.trim_end,
        retime,
        transform: build_transform_from_params(&base.params),
        animations: base.animations.clone(),
        opacity: read_opacity_from_params(&base.params),
        blend_mode: read_blend_mode_from_params(&base.params),
        effects,
        masks,
    }
}

fn build_track_nodes(
    tracks: &[&Track],
    media_map: &HashMap<&str, &MediaSource>,
    canvas_size: CanvasSize,
    is_preview: bool,
) -> Vec<Node> {
    let mut nodes = Vec::new();

    for track in tracks {
        for element in visible_sorted_elements(track) {
            match element {
                Element::Effect(effect) => {
                    nodes.push(Node::EffectLayer(EffectLayerParams {
                        effect_type: effect.effect_type.clone(),
                        effect_params: effect.base.params.clone(),
                        time_offset: effect.base.start_time,
                        duration: effect.base.duration,
                    }));
                }
                Element::Video(video) => {
                    let Some(asset) = media_map.get(video.media_id.as_str()) else {
                        continue;
                    };
                    if asset.media_type != MediaType::Video {
                        continue;
                    }
                    nodes.push(Node::Video {
                        visual: visual_params(element),
                        media_id: asset.id.clone(),
                        path: asset.path.clone(),
                    });
                }
                Element::Image(image) => {
                    let Some(asset) = media_map.get(image.media_id.as_str()) else {
                        continue;
                    };
                    if asset.media_type != MediaType::Image {
                        continue;
                    }
                    nodes.push(Node::Image {
                        visual: visual_params(element),
                        path: asset.path.clone(),
                        max_source_size: is_preview.then_some(PREVIEW_MAX_IMAGE_SIZE),
                    });
                }
                Element::Text(text) => {
                    nodes.push(Node::Text(TextNodeParams {
                        element: text.clone(),
                        transform: build_transform_from_params(&text.base.params),
                        opacity: read_opacity_from_params(&text.base.params),
                        blend_mode: read_blend_mode_from_params(&text.base.params),
                        canvas_center_x: canvas_size.width as f64 / 2.0,
                        canvas_center_y: canvas_size.height as f64 / 2.0,
                        canvas_height: canvas_size.height as f64,
                        effects: text.effects.clone().unwrap_or_default(),
                    }));
                }
                Element::Sticker(sticker) => {
                    nodes.push(Node::Sticker(StickerNodeParams {
                        visual: visual_params(element),
                        sticker_id: sticker.sticker_id.clone(),
                        intrinsic_width: sticker.intrinsic_width,
                        intrinsic_height: sticker.intrinsic_height,
                    }));
                }
                Element::Graphic(graphic) => {
                    nodes.push(Node::Graphic(GraphicNodeParams {
                        visual: visual_params(element),
                        definition_id: graphic.definition_id.clone(),
                        params: graphic.base.params.clone(),
                    }));
                }
                Element::Audio(_) => {}
            }
        }
    }

    nodes
}

fn build_blur_background_nodes(
    main_track: Option<&Track>,
    media_map: &HashMap<&str, &MediaSource>,
    blur_intensity: f64,
) -> Vec<Node> {
    let Some(track) = main_track else {
        return Vec::new();
    };

    let mut nodes = Vec::new();
    for element in visible_sorted_elements(track) {
        let (media_id, retime) = match element {
            Element::Video(video) => (&video.media_id, video.retime),
            Element::Image(image) => (&image.media_id, None),
            _ => continue,
        };
        let Some(asset) = media_map.get(media_id.as_str()) else {
            continue;
        };
        if !matches!(asset.media_type, MediaType::Video | MediaType::Image) {
            continue;
        }
        let base = element.base();
        nodes.push(Node::BlurBackground(BlurBackgroundParams {
            media_id: asset.id.clone(),
            path: asset.path.clone(),
            media_type: asset.media_type,
            duration: base.duration,
            time_offset: base.start_time,
            trim_start: base.trim_start,
            trim_end: base.trim_end,
            retime,
            blur_intensity,
        }));
    }
    nodes
}

pub fn build_scene(params: &BuildSceneParams) -> Node {
    let media_map: HashMap<&str, &MediaSource> =
        params.media.iter().map(|m| (m.id.as_str(), m)).collect();

    let mut visible_tracks: Vec<&Track> = params
        .tracks
        .overlay
        .iter()
        .filter(|track| !track_hidden(track))
        .collect();
    if !track_hidden(&params.tracks.main) {
        visible_tracks.push(&params.tracks.main);
    }
    visible_tracks.reverse();

    let mut children = Vec::new();

    if let Background::Blur { blur_intensity } = params.background {
        let main_track = (!track_hidden(&params.tracks.main)).then_some(&params.tracks.main);
        children.extend(build_blur_background_nodes(
            main_track,
            &media_map,
            *blur_intensity,
        ));
    } else if let Background::Color { color } = params.background {
        if color != "transparent" {
            children.push(Node::Color {
                color: color.clone(),
            });
        }
    }

    children.extend(build_track_nodes(
        &visible_tracks,
        &media_map,
        params.canvas_size,
        params.is_preview,
    ));

    Node::Root {
        duration: params.duration,
        children,
    }
}

fn track_hidden(track: &Track) -> bool {
    match track {
        Track::Video { hidden, .. }
        | Track::Text { hidden, .. }
        | Track::Graphic { hidden, .. }
        | Track::Effect { hidden, .. } => *hidden,
        Track::Audio { .. } => false,
    }
}
