use crate::{BVoxValuePoolValue, VoxValue};
use branded_id::soa::IdList;

/// The kind of a [`VoxValuePool`](crate::VoxValuePool) and its typed values,
/// keyed by value id.
///
/// Readers go through [`VoxValuePool::values`](crate::VoxValuePool::values).
#[derive(Clone, Debug)]
pub enum VoxValuePoolKind {
    /// Boolean values.
    Bool(IdList<BVoxValuePoolValue, bool>),

    /// Float values: finite numbers or the infinities.
    Float(IdList<BVoxValuePoolValue, f64>),

    /// Int values: magnitude at most `2^53 - 1`.
    Int(IdList<BVoxValuePoolValue, i64>),

    /// Arbitrary [`VoxValue`]s, including null.
    Json(IdList<BVoxValuePoolValue, VoxValue>),

    /// String values.
    String(IdList<BVoxValuePoolValue, String>),

    /// Two-component float vectors.
    Vec2Float(IdList<BVoxValuePoolValue, [f64; 2]>),

    /// Two-component int vectors.
    Vec2Int(IdList<BVoxValuePoolValue, [i64; 2]>),

    /// Three-component float vectors.
    Vec3Float(IdList<BVoxValuePoolValue, [f64; 3]>),

    /// Three-component int vectors.
    Vec3Int(IdList<BVoxValuePoolValue, [i64; 3]>),

    /// Four-component float vectors.
    Vec4Float(IdList<BVoxValuePoolValue, [f64; 4]>),

    /// Four-component int vectors.
    Vec4Int(IdList<BVoxValuePoolValue, [i64; 4]>),
}
