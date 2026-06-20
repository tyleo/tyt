#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A palette attribute value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VoxjAttrValue {
    Number(f64),
    Text(String),
    Bool(bool),
}

impl From<f64> for VoxjAttrValue {
    fn from(v: f64) -> Self {
        VoxjAttrValue::Number(v)
    }
}

impl From<String> for VoxjAttrValue {
    fn from(v: String) -> Self {
        VoxjAttrValue::Text(v)
    }
}
