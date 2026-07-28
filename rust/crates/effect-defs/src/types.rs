use std::collections::HashMap;

use effects::{EffectPass, UniformValue};
use scene::params::ParamValues;

use crate::params::ParamDefinition;

pub struct PassContext<'a> {
    pub effect_params: &'a ParamValues,
    pub width: u32,
    pub height: u32,
}

pub struct EffectPassTemplate {
    pub shader: &'static str,
    pub uniforms: fn(&PassContext) -> HashMap<String, UniformValue>,
}

pub struct EffectRendererConfig {
    pub passes: Vec<EffectPassTemplate>,
    pub build_passes: Option<fn(&PassContext) -> Vec<EffectPass>>,
}

pub struct EffectDefinition {
    pub effect_type: &'static str,
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub params: Vec<ParamDefinition>,
    pub renderer: EffectRendererConfig,
}
