use effect_defs::{
    EffectPassTemplate, EffectRendererConfig, EffectsRegistry, GAUSSIAN_BLUR_SHADER,
    ParamDefinition, blur_effect_definition, build_default_param_values,
    build_gaussian_blur_passes, intensity_to_sigma, params_with_defaults, parse_intensity,
    register_default_effects, resolve_effect_passes,
};
use effects::{EffectPass, UniformValue};
use scene::params::{ParamValue, ParamValues};

fn number_uniform(pass: &EffectPass, key: &str) -> f32 {
    match pass.uniforms.get(key) {
        Some(UniformValue::Number(value)) => *value,
        other => panic!("expected number uniform {key}, got {other:?}"),
    }
}

fn vector_uniform(pass: &EffectPass, key: &str) -> Vec<f32> {
    match pass.uniforms.get(key) {
        Some(UniformValue::Vector(value)) => value.clone(),
        other => panic!("expected vector uniform {key}, got {other:?}"),
    }
}

fn params_with_intensity(value: f64) -> ParamValues {
    ParamValues::from([("intensity".to_string(), ParamValue::Number(value))])
}

fn assert_close(actual: f32, expected: f64) {
    assert!(
        (actual as f64 - expected).abs() < 1e-5,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn intensity_to_sigma_scales_by_resolution_ratio() {
    assert_eq!(intensity_to_sigma(15.0, 1920.0, 1920.0), 3.0);
    assert_eq!(intensity_to_sigma(10.0, 960.0, 1920.0), 1.0);
    assert_eq!(intensity_to_sigma(0.0, 1920.0, 1920.0), 0.0);
    assert_eq!(intensity_to_sigma(100.0, 1080.0, 1080.0), 20.0);
}

#[test]
fn blur_passes_below_threshold_sigma_are_empty() {
    assert!(build_gaussian_blur_passes(0.0, 0.0).is_empty());
    assert!(build_gaussian_blur_passes(0.0009, 0.0009).is_empty());
    assert!(build_gaussian_blur_passes(f64::NAN, 3.0).is_empty());
}

#[test]
fn blur_passes_single_iteration_at_default_intensity() {
    let passes = build_gaussian_blur_passes(3.0, 3.0);
    assert_eq!(passes.len(), 2);

    assert_eq!(passes[0].shader, GAUSSIAN_BLUR_SHADER);
    assert_close(number_uniform(&passes[0], "u_sigma"), 3.0);
    assert_close(number_uniform(&passes[0], "u_step"), 1.0);
    assert_eq!(vector_uniform(&passes[0], "u_direction"), vec![1.0, 0.0]);

    assert_eq!(passes[1].shader, GAUSSIAN_BLUR_SHADER);
    assert_close(number_uniform(&passes[1], "u_sigma"), 3.0);
    assert_close(number_uniform(&passes[1], "u_step"), 1.0);
    assert_eq!(vector_uniform(&passes[1], "u_direction"), vec![0.0, 1.0]);
}

#[test]
fn blur_passes_step_up_when_sigma_exceeds_single_pass_limit() {
    let passes = build_gaussian_blur_passes(20.0, 20.0);
    assert_eq!(passes.len(), 2);
    assert_close(number_uniform(&passes[0], "u_sigma"), 20.0);
    assert_close(number_uniform(&passes[0], "u_step"), 2.0);
    assert_close(number_uniform(&passes[1], "u_step"), 2.0);
}

#[test]
fn blur_passes_iterate_for_large_sigma() {
    let passes = build_gaussian_blur_passes(50.0, 50.0);
    assert_eq!(passes.len(), 4);
    assert_close(number_uniform(&passes[0], "u_sigma"), 35.35533905932738);
    assert_close(number_uniform(&passes[0], "u_step"), 3.5355339059327378);
    assert_close(number_uniform(&passes[2], "u_sigma"), 35.35533905932738);
}

#[test]
fn blur_passes_cap_at_max_iterations() {
    let passes = build_gaussian_blur_passes(400.0, 400.0);
    assert_eq!(passes.len(), 16);
    assert_close(number_uniform(&passes[0], "u_sigma"), 141.4213562373095);
    assert_close(number_uniform(&passes[0], "u_step"), 14.14213562373095);
}

#[test]
fn blur_passes_support_anisotropic_sigma() {
    let passes = build_gaussian_blur_passes(3.0, 0.0);
    assert_eq!(passes.len(), 2);
    assert_close(number_uniform(&passes[0], "u_sigma"), 3.0);
    assert_close(number_uniform(&passes[1], "u_sigma"), 0.0);
    assert_close(number_uniform(&passes[1], "u_step"), 1.0);
}

#[test]
fn parse_intensity_matches_js_coercion() {
    assert_eq!(parse_intensity(&params_with_intensity(25.0)), 25.0);
    assert_eq!(
        parse_intensity(&ParamValues::from([(
            "intensity".to_string(),
            ParamValue::String("25".to_string())
        )])),
        25.0
    );
    assert_eq!(
        parse_intensity(&ParamValues::from([(
            "intensity".to_string(),
            ParamValue::String("25abc".to_string())
        )])),
        25.0
    );
    assert!(
        parse_intensity(&ParamValues::from([(
            "intensity".to_string(),
            ParamValue::Bool(true)
        )]))
        .is_nan()
    );
    assert!(parse_intensity(&ParamValues::new()).is_nan());
    assert!(
        parse_intensity(&ParamValues::from([(
            "intensity".to_string(),
            ParamValue::String("abc".to_string())
        )]))
        .is_nan()
    );
}

#[test]
fn blur_definition_matches_web_schema() {
    let definition = blur_effect_definition();
    assert_eq!(definition.effect_type, "blur");
    assert_eq!(definition.name, "Blur");
    assert_eq!(definition.keywords, ["blur", "soft", "defocus"]);
    assert_eq!(definition.params.len(), 1);

    let ParamDefinition::Number(intensity) = &definition.params[0] else {
        panic!("expected number param");
    };
    assert_eq!(intensity.key, "intensity");
    assert_eq!(intensity.label, "Intensity");
    assert_eq!(intensity.default_value, 15.0);
    assert_eq!(intensity.min, 0.0);
    assert_eq!(intensity.max, Some(100.0));
    assert_eq!(intensity.step, 1.0);

    assert_eq!(definition.renderer.passes.len(), 2);
    assert!(definition.renderer.build_passes.is_some());
    for template in &definition.renderer.passes {
        assert_eq!(template.shader, GAUSSIAN_BLUR_SHADER);
    }
}

#[test]
fn registry_resolves_definitions_by_type() {
    let mut registry = EffectsRegistry::new();
    register_default_effects(&mut registry);
    register_default_effects(&mut registry);

    assert!(registry.has("blur"));
    assert!(!registry.has("unknown"));
    assert_eq!(registry.get_all().len(), 1);
    assert_eq!(registry.get("blur").unwrap().name, "Blur");

    let error = registry.get("unknown").unwrap_err();
    assert_eq!(error.to_string(), "Unknown effect: unknown");
}

#[test]
fn default_param_values_come_from_definition() {
    let definition = blur_effect_definition();
    let defaults = build_default_param_values(&definition.params);
    assert_eq!(
        defaults.get("intensity"),
        Some(&ParamValue::Number(15.0))
    );

    let merged = params_with_defaults(&definition, &ParamValues::new());
    assert_eq!(merged.get("intensity"), Some(&ParamValue::Number(15.0)));

    let overridden = params_with_defaults(&definition, &params_with_intensity(42.0));
    assert_eq!(
        overridden.get("intensity"),
        Some(&ParamValue::Number(42.0))
    );
}

#[test]
fn resolve_effect_passes_prefers_build_passes() {
    let definition = blur_effect_definition();
    let passes = resolve_effect_passes(&definition, &params_with_intensity(15.0), 1920, 1080);
    assert_eq!(passes.len(), 2);
    assert_close(number_uniform(&passes[0], "u_sigma"), 3.0);
    assert_close(number_uniform(&passes[1], "u_sigma"), 3.0);
    assert_eq!(vector_uniform(&passes[0], "u_direction"), vec![1.0, 0.0]);
    assert_eq!(vector_uniform(&passes[1], "u_direction"), vec![0.0, 1.0]);

    let stepped = resolve_effect_passes(&definition, &params_with_intensity(100.0), 1920, 1080);
    assert_eq!(stepped.len(), 2);
    assert_close(number_uniform(&stepped[0], "u_sigma"), 20.0);
    assert_close(number_uniform(&stepped[0], "u_step"), 2.0);
}

#[test]
fn resolve_effect_passes_with_defaults_applied_matches_explicit_params() {
    let definition = blur_effect_definition();
    let merged = params_with_defaults(&definition, &ParamValues::new());
    let from_defaults = resolve_effect_passes(&definition, &merged, 1920, 1080);
    let explicit = resolve_effect_passes(&definition, &params_with_intensity(15.0), 1920, 1080);
    assert_eq!(from_defaults.len(), explicit.len());
    assert_close(
        number_uniform(&from_defaults[0], "u_sigma"),
        number_uniform(&explicit[0], "u_sigma") as f64,
    );
}

#[test]
fn resolve_effect_passes_without_intensity_produces_no_passes() {
    let definition = blur_effect_definition();
    let passes = resolve_effect_passes(&definition, &ParamValues::new(), 1920, 1080);
    assert!(passes.is_empty());
}

#[test]
fn resolve_effect_passes_falls_back_to_pass_templates() {
    let mut definition = blur_effect_definition();
    definition.renderer.build_passes = None;

    let passes = resolve_effect_passes(&definition, &params_with_intensity(15.0), 1920, 1080);
    assert_eq!(passes.len(), 2);
    assert_close(number_uniform(&passes[0], "u_sigma"), 3.0);
    assert_close(number_uniform(&passes[0], "u_step"), 1.0);
    assert_eq!(vector_uniform(&passes[0], "u_direction"), vec![1.0, 0.0]);
    assert_close(number_uniform(&passes[1], "u_sigma"), 3.0);
    assert_eq!(vector_uniform(&passes[1], "u_direction"), vec![0.0, 1.0]);

    let zero = resolve_effect_passes(&definition, &params_with_intensity(0.0), 1920, 1080);
    assert_eq!(zero.len(), 2);
    assert_close(number_uniform(&zero[0], "u_sigma"), 0.001);
    assert_close(number_uniform(&zero[1], "u_sigma"), 0.001);
}

#[test]
fn template_path_scales_sigma_with_resolution() {
    let definition = blur_effect_definition();
    let renderer = EffectRendererConfig {
        passes: vec![EffectPassTemplate {
            shader: GAUSSIAN_BLUR_SHADER,
            uniforms: definition.renderer.passes[0].uniforms,
        }],
        build_passes: None,
    };
    let template_only = effect_defs::EffectDefinition {
        effect_type: definition.effect_type,
        name: definition.name,
        keywords: definition.keywords,
        params: definition.params.clone(),
        renderer,
    };

    let passes = resolve_effect_passes(&template_only, &params_with_intensity(10.0), 960, 540);
    assert_eq!(passes.len(), 1);
    assert_close(number_uniform(&passes[0], "u_sigma"), 1.0);
}
