use crate::VoxjMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};

/// An arbitrary Voxel Json value: the data model shared by palette cell values
/// and the opaque `main.ext` namespace.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VoxjValue {
    /// A number, held as `f64` but serialized as a JSON integer when integral,
    /// so a value like `4` does not round-trip as `4.0`.
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
            // Serialize an integral number as a JSON integer.
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
