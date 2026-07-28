use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StrokeAlign {
    Inside,
    Center,
    Outside,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFontWeight {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "bold")]
    Bold,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFontStyle {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "italic")]
    Italic,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextDecoration {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "underline")]
    Underline,
    #[serde(rename = "line-through")]
    LineThrough,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FreeformPathPoint {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub in_x: f64,
    pub in_y: f64,
    pub out_x: f64,
    pub out_y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SplitMaskParams {
    pub feather: f64,
    pub inverted: bool,
    pub stroke_color: String,
    pub stroke_width: f64,
    pub stroke_align: StrokeAlign,
    pub center_x: f64,
    pub center_y: f64,
    pub rotation: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RectangleMaskParams {
    pub feather: f64,
    pub inverted: bool,
    pub stroke_color: String,
    pub stroke_width: f64,
    pub stroke_align: StrokeAlign,
    pub center_x: f64,
    pub center_y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub scale: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextMaskParams {
    pub feather: f64,
    pub inverted: bool,
    pub stroke_color: String,
    pub stroke_width: f64,
    pub stroke_align: StrokeAlign,
    pub content: String,
    pub font_size: f64,
    pub font_family: String,
    pub font_weight: TextFontWeight,
    pub font_style: TextFontStyle,
    pub text_decoration: TextDecoration,
    pub letter_spacing: f64,
    pub line_height: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub rotation: f64,
    pub scale: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FreeformPathMaskParams {
    pub feather: f64,
    pub inverted: bool,
    pub stroke_color: String,
    pub stroke_width: f64,
    pub stroke_align: StrokeAlign,
    pub path: Vec<FreeformPathPoint>,
    pub closed: bool,
    pub center_x: f64,
    pub center_y: f64,
    pub rotation: f64,
    pub scale: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Mask {
    Split { id: String, params: SplitMaskParams },
    CinematicBars { id: String, params: RectangleMaskParams },
    Rectangle { id: String, params: RectangleMaskParams },
    Ellipse { id: String, params: RectangleMaskParams },
    Heart { id: String, params: RectangleMaskParams },
    Diamond { id: String, params: RectangleMaskParams },
    Star { id: String, params: RectangleMaskParams },
    Text { id: String, params: TextMaskParams },
    Freeform { id: String, params: FreeformPathMaskParams },
}
