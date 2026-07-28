use std::collections::HashMap;

use crate::{EffectDefsError, types::EffectDefinition};

#[derive(Default)]
pub struct EffectsRegistry {
    definitions: HashMap<String, EffectDefinition>,
}

impl EffectsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, definition: EffectDefinition) {
        self.definitions
            .insert(definition.effect_type.to_string(), definition);
    }

    pub fn has(&self, key: &str) -> bool {
        self.definitions.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Result<&EffectDefinition, EffectDefsError> {
        self.definitions
            .get(key)
            .ok_or_else(|| EffectDefsError::UnknownEffect {
                effect_type: key.to_string(),
            })
    }

    pub fn get_all(&self) -> Vec<&EffectDefinition> {
        self.definitions.values().collect()
    }
}
