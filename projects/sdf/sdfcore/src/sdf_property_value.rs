use crate::SdfValue;

/// A material property's value. The variant sets the kind a custom property
/// takes.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfPropertyValue {
    /// A boolean.
    Bool(bool),

    /// An `int` of one number.
    Int(f64),

    /// An `int` of a list of numbers.
    IntArray(Vec<f64>),

    /// A `json` value.
    Json(SdfValue),

    /// A number.
    Number(f64),

    /// A list of numbers.
    NumberArray(Vec<f64>),

    /// A string.
    Text(String),
}
