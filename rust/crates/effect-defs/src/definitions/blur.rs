use std::collections::HashMap;

use effects::{EffectPass, UniformValue};
use scene::params::{ParamValue, ParamValues};

use crate::math::{js_max, js_min, js_parse_float};
use crate::params::{NumberParamDefinition, ParamDefinition};
use crate::types::{EffectDefinition, EffectPassTemplate, EffectRendererConfig, PassContext};

pub const GAUSSIAN_BLUR_SHADER: &str = "gaussian-blur";

const MAX_SINGLE_PASS_SIGMA: f64 = 10.0;
const MAX_STEP: f64 = 4.0;
const MAX_EFFECTIVE_SIGMA: f64 = MAX_SINGLE_PASS_SIGMA * MAX_STEP;
const MAX_ITERATIONS: f64 = 8.0;

pub const INTENSITY_TO_SIGMA_DIVISOR: f64 = 5.0;

pub fn build_gaussian_blur_passes(sigma_x: f64, sigma_y: f64) -> Vec<EffectPass> {
    let max_sigma = js_max(sigma_x, sigma_y);
    if max_sigma < 0.001 {
        return Vec::new();
    }

    let iterations = js_min(
        MAX_ITERATIONS,
        js_max(
            1.0,
            ((max_sigma * max_sigma) / (MAX_EFFECTIVE_SIGMA * MAX_EFFECTIVE_SIGMA)).ceil(),
        ),
    );
    let iteration_count = iterations.max(0.0) as usize;
    let per_pass_sigma_x = sigma_x / iterations.sqrt();
    let per_pass_sigma_y = sigma_y / iterations.sqrt();
    let step_x = js_max(1.0, per_pass_sigma_x / MAX_SINGLE_PASS_SIGMA);
    let step_y = js_max(1.0, per_pass_sigma_y / MAX_SINGLE_PASS_SIGMA);

    let mut passes = Vec::with_capacity(iteration_count * 2);
    for _ in 0..iteration_count {
        passes.push(blur_pass(per_pass_sigma_x, step_x, [1.0, 0.0]));
        passes.push(blur_pass(per_pass_sigma_y, step_y, [0.0, 1.0]));
    }
    passes
}

pub fn intensity_to_sigma(intensity: f64, resolution: f64, reference: f64) -> f64 {
    (intensity / INTENSITY_TO_SIGMA_DIVISOR) * (resolution / reference)
}

pub fn parse_intensity(effect_params: &ParamValues) -> f64 {
    match effect_params.get("intensity") {
        Some(ParamValue::Number(value)) => *value,
        Some(ParamValue::String(value)) => js_parse_float(value),
        Some(ParamValue::Bool(true)) => js_parse_float("true"),
        Some(ParamValue::Bool(false)) => js_parse_float("false"),
        None => js_parse_float("undefined"),
    }
}

pub fn blur_effect_definition() -> EffectDefinition {
    EffectDefinition {
        effect_type: "blur",
        name: "Blur",
        keywords: &["blur", "soft", "defocus"],
        params: vec![ParamDefinition::Number(NumberParamDefinition::new(
            "intensity",
            "Intensity",
            15.0,
            0.0,
            Some(100.0),
            1.0,
        ))],
        renderer: EffectRendererConfig {
            passes: vec![
                EffectPassTemplate {
                    shader: GAUSSIAN_BLUR_SHADER,
                    uniforms: horizontal_template_uniforms,
                },
                EffectPassTemplate {
                    shader: GAUSSIAN_BLUR_SHADER,
                    uniforms: vertical_template_uniforms,
                },
            ],
            build_passes: Some(build_passes_from_params),
        },
    }
}

fn blur_pass(sigma: f64, step: f64, direction: [f32; 2]) -> EffectPass {
    EffectPass {
        shader: GAUSSIAN_BLUR_SHADER.to_string(),
        uniforms: HashMap::from([
            ("u_sigma".to_string(), UniformValue::Number(sigma as f32)),
            ("u_step".to_string(), UniformValue::Number(step as f32)),
            (
                "u_direction".to_string(),
                UniformValue::Vector(direction.to_vec()),
            ),
        ]),
    }
}

fn horizontal_template_uniforms(context: &PassContext) -> HashMap<String, UniformValue> {
    let sigma = js_max(
        intensity_to_sigma(
            parse_intensity(context.effect_params),
            context.width as f64,
            1920.0,
        ),
        0.001,
    );
    HashMap::from([
        ("u_sigma".to_string(), UniformValue::Number(sigma as f32)),
        ("u_step".to_string(), UniformValue::Number(1.0)),
        (
            "u_direction".to_string(),
            UniformValue::Vector(vec![1.0, 0.0]),
        ),
    ])
}

fn vertical_template_uniforms(context: &PassContext) -> HashMap<String, UniformValue> {
    let sigma = js_max(
        intensity_to_sigma(
            parse_intensity(context.effect_params),
            context.height as f64,
            1080.0,
        ),
        0.001,
    );
    HashMap::from([
        ("u_sigma".to_string(), UniformValue::Number(sigma as f32)),
        ("u_step".to_string(), UniformValue::Number(1.0)),
        (
            "u_direction".to_string(),
            UniformValue::Vector(vec![0.0, 1.0]),
        ),
    ])
}

fn build_passes_from_params(context: &PassContext) -> Vec<EffectPass> {
    let intensity = parse_intensity(context.effect_params);
    build_gaussian_blur_passes(
        intensity_to_sigma(intensity, context.width as f64, 1920.0),
        intensity_to_sigma(intensity, context.height as f64, 1080.0),
    )
}
