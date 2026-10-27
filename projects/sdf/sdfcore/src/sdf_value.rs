use crate::SdfMap;

/// An arbitrary JSON value: what a `json` property holds.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfValue {
    /// An ordered list of values.
    Array(Vec<SdfValue>),

    /// A boolean.
    Bool(bool),

    /// A null value.
    Null,

    /// A number.
    Number(f64),

    /// An ordered set of key/value pairs.
    Object(SdfMap),

    /// A string.
    Text(String),
}
