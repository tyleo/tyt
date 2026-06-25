use crate::VoxMap;

/// An arbitrary voxel value.
///
/// This is the in-memory data model shared by palette cell values and the
/// opaque extension namespace. Numbers, integral or not, are held as `f64`.
#[derive(Clone, Debug, PartialEq)]
pub enum VoxValue {
    /// A number.
    Number(f64),

    /// A string.
    Text(String),

    /// A boolean.
    Bool(bool),

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
