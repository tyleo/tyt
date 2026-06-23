use crate::VoxjMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};

/// An arbitrary Voxel Json value.
///
/// This is the JSON data model shared by palette cell values and the opaque
/// `main.ext` extension namespace. Numbers, integral or not, are held as `f64`;
/// an integral value is serialized as a JSON integer so consumers that expect
/// one round-trip without seeing a fractional `.0`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VoxjValue {
    /// A number.
    Number(f64),

    /// A string.
    Text(String),

    /// A boolean.
    Bool(bool),

    /// An ordered list of values.
    Array(Vec<VoxjValue>),

    /// An ordered set of key/value pairs.
    Object(VoxjMap),

    /// JSON `null`.
    Null,
}

#[cfg(feature = "serde")]
impl Serialize for VoxjValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Integral numbers serialize as a JSON integer, not a fractional `4.0`.
            VoxjValue::Number(n)
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 =>
            {
                serializer.serialize_i64(*n as i64)
            }

            VoxjValue::Number(n) => serializer.serialize_f64(*n),

            VoxjValue::Text(text) => serializer.serialize_str(text),

            VoxjValue::Bool(bool) => serializer.serialize_bool(*bool),

            VoxjValue::Array(array) => array.serialize(serializer),

            VoxjValue::Object(object) => object.serialize(serializer),

            VoxjValue::Null => serializer.serialize_unit(),
        }
    }
}

impl From<f64> for VoxjValue {
    fn from(v: f64) -> Self {
        VoxjValue::Number(v)
    }
}

impl From<String> for VoxjValue {
    fn from(v: String) -> Self {
        VoxjValue::Text(v)
    }
}
