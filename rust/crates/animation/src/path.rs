use crate::effect_param_channel::is_effect_param_path;
use crate::graphic_param_channel::is_graphic_param_path;
use crate::property_groups::is_animation_property_path;

pub fn is_animation_path(property_path: &str) -> bool {
    is_animation_property_path(property_path)
        || is_graphic_param_path(property_path)
        || is_effect_param_path(property_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_animation_paths() {
        assert!(is_animation_path("opacity"));
        assert!(is_animation_path("params.blur"));
        assert!(is_animation_path("effects.glow.params.intensity"));
        assert!(!is_animation_path("transform"));
        assert!(!is_animation_path("bindings"));
    }
}
