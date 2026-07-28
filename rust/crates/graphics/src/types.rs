use scene::params::{ParamValue, ParamValues};
use tiny_skia::Pixmap;

use crate::error::GraphicsError;
use crate::params::ParamDefinition;

pub const DEFAULT_GRAPHIC_SOURCE_SIZE: u32 = 512;

pub struct GraphicRenderContext<'a> {
    pub pixmap: &'a mut Pixmap,
    pub params: &'a ParamValues,
    pub width: u32,
    pub height: u32,
}

pub struct GraphicDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub params: &'static [ParamDefinition],
    pub render: fn(&mut GraphicRenderContext) -> Result<(), GraphicsError>,
}

impl GraphicDefinition {
    pub fn render_to_pixmap(
        &self,
        params: &ParamValues,
        width: u32,
        height: u32,
    ) -> Result<Pixmap, GraphicsError> {
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| GraphicsError::InvalidColor("invalid size".to_string()))?;
        let mut context = GraphicRenderContext {
            pixmap: &mut pixmap,
            params,
            width,
            height,
        };
        (self.render)(&mut context)?;
        Ok(pixmap)
    }

    pub fn render_rgba8(
        &self,
        params: &ParamValues,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, GraphicsError> {
        Ok(self.render_to_pixmap(params, width, height)?.take())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicInstance {
    pub definition_id: String,
    pub params: ParamValues,
}

pub fn param_string(params: &ParamValues, key: &str, default: &str) -> String {
    match params.get(key) {
        Some(ParamValue::String(value)) => value.clone(),
        Some(ParamValue::Number(value)) => value.to_string(),
        Some(ParamValue::Bool(value)) => value.to_string(),
        None => default.to_string(),
    }
}

pub fn param_number(params: &ParamValues, key: &str, default: f64) -> f64 {
    match params.get(key) {
        Some(ParamValue::Number(value)) => *value,
        Some(ParamValue::String(value)) => value.trim().parse::<f64>().unwrap_or(default),
        Some(ParamValue::Bool(value)) => {
            if *value { 1.0 } else { 0.0 }
        }
        None => default,
    }
}
