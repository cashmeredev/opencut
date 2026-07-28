mod definitions;
mod math;
mod params;
mod registry;
mod types;

use effects::EffectPass;
use scene::ParamValues;
use thiserror::Error;

pub use definitions::blur::{
    GAUSSIAN_BLUR_SHADER, INTENSITY_TO_SIGMA_DIVISOR, blur_effect_definition,
    build_gaussian_blur_passes, intensity_to_sigma, parse_intensity,
};
pub use definitions::register_default_effects;
pub use params::{
    BooleanParamDefinition, ColorParamDefinition, FontParamDefinition, NumberParamDefinition,
    ParamDefinition, ParamDependency, SelectOption, SelectParamDefinition, TextParamDefinition,
    build_default_param_values,
};
pub use registry::EffectsRegistry;
pub use types::{EffectDefinition, EffectPassTemplate, EffectRendererConfig, PassContext};

#[derive(Debug, Error, PartialEq)]
pub enum EffectDefsError {
    #[error("Unknown effect: {effect_type}")]
    UnknownEffect { effect_type: String },
}

pub fn resolve_effect_passes(
    definition: &EffectDefinition,
    effect_params: &ParamValues,
    width: u32,
    height: u32,
) -> Vec<EffectPass> {
    let context = PassContext {
        effect_params,
        width,
        height,
    };
    if let Some(build_passes) = definition.renderer.build_passes {
        return build_passes(&context);
    }
    definition
        .renderer
        .passes
        .iter()
        .map(|pass| EffectPass {
            shader: pass.shader.to_string(),
            uniforms: (pass.uniforms)(&context),
        })
        .collect()
}

pub fn params_with_defaults(
    definition: &EffectDefinition,
    effect_params: &ParamValues,
) -> ParamValues {
    let mut values = build_default_param_values(&definition.params);
    for (key, value) in effect_params {
        values.insert(key.clone(), value.clone());
    }
    values
}
