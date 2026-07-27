use crate::VoxValue;

/// One value read out of a [`VoxValuePool`](crate::VoxValuePool).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxPoolValueRef<'a> {
    /// A value from a `json` pool.
    Json(&'a VoxValue),

    /// A value from a `bool` pool.
    Bool(bool),

    /// A value from a `float` pool.
    Float(f64),

    /// A value from an `int` pool.
    Int(i64),

    /// A value from a `string` pool.
    String(&'a str),

    /// A color from an `srgb` pool.
    Srgb(&'a [f64; 3]),

    /// A color from an `srgba` pool.
    Srgba(&'a [f64; 4]),

    /// A color from a `linear-rgb` pool.
    LinearRgb(&'a [f64; 3]),

    /// A color from a `linear-rgba` pool.
    LinearRgba(&'a [f64; 4]),
}
