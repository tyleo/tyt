use crate::{VoxValue, VoxValueColumn};

/// A [`VoxValuePool`](crate::VoxValuePool)'s values, typed by its kind. Match
/// once, then read the column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxValuePoolValues<'a> {
    /// A `bool` value pool's values.
    Bool(VoxValueColumn<'a, bool>),

    /// A `float` value pool's values: finite numbers or the infinities.
    Float(VoxValueColumn<'a, f64>),

    /// An `int` value pool's values: magnitude at most `2^53 - 1`.
    Int(VoxValueColumn<'a, i64>),

    /// A `json` value pool's values, including null.
    Json(VoxValueColumn<'a, VoxValue>),

    /// A `string` value pool's values.
    String(VoxValueColumn<'a, String>),

    /// A `vec-2-float` value pool's values.
    Vec2Float(VoxValueColumn<'a, [f64; 2]>),

    /// A `vec-2-int` value pool's values.
    Vec2Int(VoxValueColumn<'a, [i64; 2]>),

    /// A `vec-3-float` value pool's values.
    Vec3Float(VoxValueColumn<'a, [f64; 3]>),

    /// A `vec-3-int` value pool's values.
    Vec3Int(VoxValueColumn<'a, [i64; 3]>),

    /// A `vec-4-float` value pool's values.
    Vec4Float(VoxValueColumn<'a, [f64; 4]>),

    /// A `vec-4-int` value pool's values.
    Vec4Int(VoxValueColumn<'a, [i64; 4]>),
}
