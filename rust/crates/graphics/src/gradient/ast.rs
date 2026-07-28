#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientKind {
    Linear,
    RepeatingLinear,
    Radial,
    RepeatingRadial,
}

impl GradientKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            GradientKind::Linear => "linear-gradient",
            GradientKind::RepeatingLinear => "repeating-linear-gradient",
            GradientKind::Radial => "radial-gradient",
            GradientKind::RepeatingRadial => "repeating-radial-gradient",
        }
    }

    pub fn is_linear(&self) -> bool {
        matches!(self, GradientKind::Linear | GradientKind::RepeatingLinear)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinearOrientation {
    Directional(String),
    Angular(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Distance {
    Percent(String),
    PositionKeyword(String),
    Calc(String),
    Px(String),
    Em(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub x: Option<Distance>,
    pub y: Option<Distance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Circle,
    Ellipse,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeStyle {
    Distance(Distance),
    ExtentKeyword(String),
    Position(Position),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub value: ShapeKind,
    pub style: Option<ShapeStyle>,
    pub at: Option<Position>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RadialOrientation {
    Shape(Shape),
    ExtentKeyword {
        value: String,
        at: Option<Position>,
    },
    DefaultRadial {
        at: Position,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GradientOrientation {
    Linear(LinearOrientation),
    Radial(Vec<RadialOrientation>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Hex(String),
    Literal(String),
    Rgb(Vec<String>),
    Rgba(Vec<String>),
    Hsl([String; 3]),
    Hsla([String; 4]),
    Var(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    pub color: Color,
    pub length: Option<Distance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GradientAst {
    pub kind: GradientKind,
    pub orientation: Option<GradientOrientation>,
    pub color_stops: Vec<ColorStop>,
}
