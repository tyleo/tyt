use crate::VoxValue;

/// One value read out of a [`VoxValuePool`](crate::VoxValuePool).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxValuePoolValueRef<'a> {
    /// A value from a `bool` value pool.
    Bool(bool),

    /// A value from a `float` value pool.
    Float(f64),

    /// A value from an `int` value pool.
    Int(i64),

    /// A value from a `json` value pool.
    Json(&'a VoxValue),

    /// A value from a `string` value pool.
    String(&'a str),

    /// A value from a `vec-2-float` value pool.
    Vec2Float(&'a [f64; 2]),

    /// A value from a `vec-2-int` value pool.
    Vec2Int(&'a [i64; 2]),

    /// A value from a `vec-3-float` value pool.
    Vec3Float(&'a [f64; 3]),

    /// A value from a `vec-3-int` value pool.
    Vec3Int(&'a [i64; 3]),

    /// A value from a `vec-4-float` value pool.
    Vec4Float(&'a [f64; 4]),

    /// A value from a `vec-4-int` value pool.
    Vec4Int(&'a [i64; 4]),
}
