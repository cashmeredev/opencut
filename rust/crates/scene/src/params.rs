use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ParamValue {
    Number(f64),
    String(String),
    Bool(bool),
}

pub type ParamValues = BTreeMap<String, ParamValue>;
