pub mod ast;
pub mod parser;
pub mod raster;
pub mod stops;

pub use ast::{
    Color, ColorStop, Distance, GradientAst, GradientKind, GradientOrientation,
    LinearOrientation, Position, RadialOrientation, Shape, ShapeKind, ShapeStyle,
};
pub use parser::{GradientParseError, parse_gradient};
pub use raster::{
    BackgroundLayer, draw_css_background, is_gradient_layer, parse_background_layers,
    split_css_layers,
};
pub use stops::{NormalizedColorStop, color_to_string, normalize_color_stops};
