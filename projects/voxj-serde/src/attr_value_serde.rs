use serde::{Deserialize, Serialize};
use voxj::AttrValue;

/// Serde-compatible parity type for [`AttrValue`]. Serializes as a bare number,
/// string, or boolean.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AttrValueSerde {
    Number(f64),
    Text(String),
    Bool(bool),
}

impl From<AttrValue> for AttrValueSerde {
    fn from(v: AttrValue) -> Self {
        match v {
            AttrValue::Number(n) => AttrValueSerde::Number(n),
            AttrValue::Text(t) => AttrValueSerde::Text(t),
            AttrValue::Bool(b) => AttrValueSerde::Bool(b),
        }
    }
}

impl From<AttrValueSerde> for AttrValue {
    fn from(v: AttrValueSerde) -> Self {
        match v {
            AttrValueSerde::Number(n) => AttrValue::Number(n),
            AttrValueSerde::Text(t) => AttrValue::Text(t),
            AttrValueSerde::Bool(b) => AttrValue::Bool(b),
        }
    }
}
