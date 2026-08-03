use crate::{BVoxValuePoolValue, VoxValue};
use branded_id::soa::IdField;

/// The kind of a [`VoxValuePool`](crate::VoxValuePool) and its typed value
/// column.
///
/// Each value column is keyed by the owning value pool's ids, so reading one
/// directly is unsafe. Read through
/// [`VoxValuePool::value`](crate::VoxValuePool::value) or
/// [`VoxValuePool::iter_values`](crate::VoxValuePool::iter_values), and match
/// this enum for the kind.
#[derive(Debug)]
pub enum VoxValuePoolKind {
    /// Boolean values.
    Bool(IdField<BVoxValuePoolValue, bool>),

    /// Float values: finite numbers or the infinities.
    Float(IdField<BVoxValuePoolValue, f64>),

    /// Int values: magnitude at most `2^53 - 1`.
    Int(IdField<BVoxValuePoolValue, i64>),

    /// Arbitrary [`VoxValue`]s, including null.
    Json(IdField<BVoxValuePoolValue, VoxValue>),

    /// String values.
    String(IdField<BVoxValuePoolValue, String>),

    /// Two-component float vectors.
    Vec2Float(IdField<BVoxValuePoolValue, [f64; 2]>),

    /// Two-component int vectors.
    Vec2Int(IdField<BVoxValuePoolValue, [i64; 2]>),

    /// Three-component float vectors.
    Vec3Float(IdField<BVoxValuePoolValue, [f64; 3]>),

    /// Three-component int vectors.
    Vec3Int(IdField<BVoxValuePoolValue, [i64; 3]>),

    /// Four-component float vectors.
    Vec4Float(IdField<BVoxValuePoolValue, [f64; 4]>),

    /// Four-component int vectors.
    Vec4Int(IdField<BVoxValuePoolValue, [i64; 4]>),
}
