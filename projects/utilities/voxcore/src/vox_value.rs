use crate::VoxMap;

/// An arbitrary voxel value.
///
/// This is the in-memory data model shared by palette cell values and the
/// opaque extension namespace. Numbers, integral or not, are held as `f64`, so
/// an integer past 2^53 loses precision. Store such values in an `int` value
/// pool.
#[derive(Clone, Debug, PartialEq)]
pub enum VoxValue {
    /// A boolean.
    Bool(bool),

    /// A number.
    Number(f64),

    /// A string.
    Text(String),

    /// An ordered list of values.
    Array(Vec<VoxValue>),

    /// An ordered set of key/value pairs.
    Object(VoxMap),

    /// A null value.
    Null,
}

impl From<f64> for VoxValue {
    fn from(value: f64) -> Self {
        VoxValue::Number(value)
    }
}

impl From<String> for VoxValue {
    fn from(value: String) -> Self {
        VoxValue::Text(value)
    }
}
