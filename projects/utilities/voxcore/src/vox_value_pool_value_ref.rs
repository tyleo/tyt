use crate::VoxValue;

/// One value read out of a [`VoxValuePool`](crate::VoxValuePool).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxValuePoolValueRef<'a> {
    /// A value from a `json` value pool.
    Json(&'a VoxValue),

    /// A value from a `bool` value pool.
    Bool(bool),

    /// A value from a `float` value pool.
    Float(f64),

    /// A value from an `int` value pool.
    Int(i64),

    /// A value from a `string` value pool.
    String(&'a str),

    /// A color from an `srgb` value pool.
    Srgb(&'a [f64; 3]),

    /// A color from an `srgba` value pool.
    Srgba(&'a [f64; 4]),

    /// A color from a `linear-rgb` value pool.
    LinearRgb(&'a [f64; 3]),

    /// A color from a `linear-rgba` value pool.
    LinearRgba(&'a [f64; 4]),
}
