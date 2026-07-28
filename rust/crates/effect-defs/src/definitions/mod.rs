pub mod blur;

use crate::registry::EffectsRegistry;

pub fn register_default_effects(registry: &mut EffectsRegistry) {
    for definition in [blur::blur_effect_definition()] {
        if registry.has(definition.effect_type) {
            continue;
        }
        registry.register(definition);
    }
}
