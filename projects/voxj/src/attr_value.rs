#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A palette attribute value. The recommended attributes are colors
/// (`#RRGGBBAA` strings) and numbers; booleans are accepted for custom keys.
/// Serializes as a bare number, string, or boolean.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum AttrValue {
    Number(f64),
    Text(String),
    Bool(bool),
}

impl From<f64> for AttrValue {
    fn from(v: f64) -> Self {
        AttrValue::Number(v)
    }
}

impl From<String> for AttrValue {
    fn from(v: String) -> Self {
        AttrValue::Text(v)
    }
}
