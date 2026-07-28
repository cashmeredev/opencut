use serde::{Deserialize, Serialize};

use crate::params::ParamValues;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Effect {
    pub id: String,
    #[serde(rename = "type")]
    pub effect_type: String,
    #[serde(default)]
    pub params: ParamValues,
    pub enabled: bool,
}
