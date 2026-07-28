pub mod color;
pub mod definitions;
pub mod error;
pub mod gradient;
pub mod params;
pub mod registry;
pub mod stroke;
pub mod types;

pub use color::{Rgba, hsl_to_rgb, is_transparent, named_color, parse_color_to_rgb, parse_css_color};
pub use definitions::{
    ELLIPSE_GRAPHIC_DEFINITION, POLYGON_GRAPHIC_DEFINITION, RECTANGLE_GRAPHIC_DEFINITION,
    STAR_GRAPHIC_DEFINITION, STROKE_ALIGN_PARAM, StrokeAlign, default_graphic_definitions,
};
pub use error::GraphicsError;
pub use gradient::{
    BackgroundLayer, Color, ColorStop, Distance, GradientAst, GradientKind, GradientOrientation,
    GradientParseError, LinearOrientation, NormalizedColorStop, Position, RadialOrientation,
    Shape, ShapeKind, ShapeStyle, color_to_string, draw_css_background, is_gradient_layer,
    normalize_color_stops, parse_background_layers, parse_gradient, split_css_layers,
};
pub use params::{ParamDefinition, ParamGroup, SelectOption, build_default_param_values};
pub use registry::{
    GraphicsRegistry, build_default_graphic_instance, get_graphic_definition, graphics_registry,
    render_graphic, render_graphic_rgba8, resolve_graphic_params,
};
pub use scene::params::{ParamValue, ParamValues};
pub use stroke::apply_aligned_stroke;
pub use types::{
    DEFAULT_GRAPHIC_SOURCE_SIZE, GraphicDefinition, GraphicInstance, GraphicRenderContext,
    param_number, param_string,
};
