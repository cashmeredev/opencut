use thiserror::Error;

use crate::gradient::GradientParseError;

#[derive(Debug, Error, PartialEq)]
pub enum GraphicsError {
    #[error("Unknown graphic: {0}")]
    UnknownGraphic(String),
    #[error("Invalid color: {0}")]
    InvalidColor(String),
    #[error(transparent)]
    GradientParse(#[from] GradientParseError),
}
