use std::collections::BTreeMap;
use std::sync::LazyLock;

use scene::params::ParamValues;

use crate::definitions::default_graphic_definitions;
use crate::error::GraphicsError;
use crate::params::build_default_param_values;
use crate::types::{GraphicDefinition, GraphicInstance};

#[derive(Default)]
pub struct GraphicsRegistry {
    definitions: BTreeMap<String, &'static GraphicDefinition>,
}

impl GraphicsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    pub fn register_defaults(&mut self) {
        for definition in default_graphic_definitions() {
            if self.has(definition.id) {
                continue;
            }
            self.register(definition.id, definition);
        }
    }

    pub fn register(&mut self, key: &str, definition: &'static GraphicDefinition) {
        self.definitions.insert(key.to_string(), definition);
    }

    pub fn has(&self, key: &str) -> bool {
        self.definitions.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Result<&'static GraphicDefinition, GraphicsError> {
        self.definitions
            .get(key)
            .copied()
            .ok_or_else(|| GraphicsError::UnknownGraphic(key.to_string()))
    }

    pub fn get_all(&self) -> Vec<&'static GraphicDefinition> {
        self.definitions.values().copied().collect()
    }
}

static REGISTRY: LazyLock<GraphicsRegistry> = LazyLock::new(GraphicsRegistry::with_defaults);

pub fn graphics_registry() -> &'static GraphicsRegistry {
    &REGISTRY
}

pub fn get_graphic_definition(
    definition_id: &str,
) -> Result<&'static GraphicDefinition, GraphicsError> {
    graphics_registry().get(definition_id)
}

pub fn build_default_graphic_instance(
    definition_id: &str,
) -> Result<GraphicInstance, GraphicsError> {
    let definition = get_graphic_definition(definition_id)?;
    Ok(GraphicInstance {
        definition_id: definition_id.to_string(),
        params: build_default_param_values(definition.params),
    })
}

pub fn resolve_graphic_params(
    definition: &GraphicDefinition,
    params: Option<&ParamValues>,
) -> ParamValues {
    let mut resolved = build_default_param_values(definition.params);
    if let Some(params) = params {
        resolved.extend(params.clone());
    }
    resolved
}

pub fn render_graphic(
    definition_id: &str,
    params: Option<&ParamValues>,
    width: u32,
    height: u32,
) -> Result<tiny_skia::Pixmap, GraphicsError> {
    let definition = get_graphic_definition(definition_id)?;
    let resolved = resolve_graphic_params(definition, params);
    definition.render_to_pixmap(&resolved, width, height)
}

pub fn render_graphic_rgba8(
    definition_id: &str,
    params: Option<&ParamValues>,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, GraphicsError> {
    Ok(render_graphic(definition_id, params, width, height)?.take())
}
