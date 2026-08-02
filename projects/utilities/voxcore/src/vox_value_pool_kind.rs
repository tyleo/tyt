use crate::{BVoxValuePoolValue, VoxValue};
use branded_id::soa::IdField;

/// The kind of a [`VoxValuePool`](crate::VoxValuePool) and its typed value
/// column.
///
/// Each `values` column is keyed by the owning value pool's ids, so reading one
/// directly is unsafe. Read through
/// [`VoxValuePool::value`](crate::VoxValuePool::value) or
/// [`VoxValuePool::iter_values`](crate::VoxValuePool::iter_values), and match
/// this enum for the kind.
#[derive(Debug)]
pub enum VoxValuePoolKind {
    /// Boolean values.
    Bool {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, bool>,
    },

    /// Float values: finite numbers or the infinities.
    Float {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, f64>,
    },

    /// Int values: magnitude at most `2^53 - 1`.
    Int {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, i64>,
    },

    /// Arbitrary [`VoxValue`]s, including null.
    Json {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, VoxValue>,
    },

    /// String values.
    String {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, String>,
    },

    /// Two-component float vectors.
    Vec2Float {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, [f64; 2]>,
    },

    /// Two-component int vectors.
    Vec2Int {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, [i64; 2]>,
    },

    /// Three-component float vectors.
    Vec3Float {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, [f64; 3]>,
    },

    /// Three-component int vectors.
    Vec3Int {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, [i64; 3]>,
    },

    /// Four-component float vectors.
    Vec4Float {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, [f64; 4]>,
    },

    /// Four-component int vectors.
    Vec4Int {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, [i64; 4]>,
    },
}
