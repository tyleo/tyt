use crate::{BVoxValuePoolValue, VoxBound, VoxValue};
use branded_id::soa::IdField;

/// The kind of a [`VoxValuePool`](crate::VoxValuePool) and its typed value
/// column.
///
/// Each `values` column is keyed by the owning value pool's ids, so reading one
/// directly is unsafe. Read through
/// [`VoxValuePool::value`](crate::VoxValuePool::value) or
/// [`VoxValuePool::iter_values`](crate::VoxValuePool::iter_values), and match
/// this enum for the kind and bounds.
#[derive(Debug)]
pub enum VoxValuePoolKind {
    /// Arbitrary [`VoxValue`]s, including null.
    Json {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, VoxValue>,
    },

    /// Boolean values.
    Bool {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, bool>,
    },

    /// Finite floating-point values within `min`/`max`.
    Float {
        /// Lower bound, a finite number or unbounded.
        min: VoxBound,

        /// Upper bound, a finite number or unbounded.
        max: VoxBound,

        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, f64>,
    },

    /// Integer values within `min`/`max`.
    Int {
        /// Lower bound, a finite number or unbounded.
        min: VoxBound,

        /// Upper bound, a finite number or unbounded.
        max: VoxBound,

        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, i64>,
    },

    /// String values.
    String {
        /// The value pool's values.
        values: IdField<BVoxValuePoolValue, String>,
    },

    /// Three-component sRGB colors, each float component in `[0, 1]`.
    Srgb {
        /// The value pool's colors.
        values: IdField<BVoxValuePoolValue, [f64; 3]>,
    },

    /// Four-component sRGB colors, each float component in `[0, 1]`.
    Srgba {
        /// The value pool's colors.
        values: IdField<BVoxValuePoolValue, [f64; 4]>,
    },

    /// Three-component linear colors, each float component `>= 0`.
    LinearRgb {
        /// The value pool's colors.
        values: IdField<BVoxValuePoolValue, [f64; 3]>,
    },

    /// Four-component linear colors, each float component `>= 0`.
    LinearRgba {
        /// The value pool's colors.
        values: IdField<BVoxValuePoolValue, [f64; 4]>,
    },
}
