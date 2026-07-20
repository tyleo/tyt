use crate::{BVoxPoolValue, VoxBound, VoxValue};
use branded_id::{IdVec, U32Id};

/// A shared pool of values. The variant is the pool's kind: the bounded kinds
/// (`float`, `int`) carry `min`/`max` and typed numeric values; every other kind
/// carries only its typed values.
///
/// Colors are stored as float components in the color space's natural range,
/// one canonical kind per space and alpha combination, so the wire format's
/// separate hex and float color encodings both map onto the matching variant
/// here. [`VoxMain::validate`](crate::VoxMain::validate) checks each value
/// against its kind and the bounds.
#[derive(Clone, Debug, PartialEq)]
pub enum VoxValuePool {
    /// Arbitrary [`VoxValue`](crate::VoxValue)s, including null.
    Json {
        /// The pooled values.
        values: IdVec<BVoxPoolValue, VoxValue>,
    },

    /// Boolean values.
    Bool {
        /// The pooled values.
        values: IdVec<BVoxPoolValue, bool>,
    },

    /// Finite floating-point values within `min`/`max`.
    Float {
        /// Lower bound, a finite number or unbounded.
        min: VoxBound,

        /// Upper bound, a finite number or unbounded.
        max: VoxBound,

        /// The pooled values.
        values: IdVec<BVoxPoolValue, f64>,
    },

    /// Integer values within `min`/`max`.
    Int {
        /// Lower bound, a finite number or unbounded.
        min: VoxBound,

        /// Upper bound, a finite number or unbounded.
        max: VoxBound,

        /// The pooled values.
        values: IdVec<BVoxPoolValue, i64>,
    },

    /// String values.
    String {
        /// The pooled values.
        values: IdVec<BVoxPoolValue, String>,
    },

    /// Three-component sRGB colors, each float component in `[0, 1]`.
    Srgb {
        /// The pooled colors.
        values: IdVec<BVoxPoolValue, [f64; 3]>,
    },

    /// Four-component sRGB colors, each float component in `[0, 1]`.
    Srgba {
        /// The pooled colors.
        values: IdVec<BVoxPoolValue, [f64; 4]>,
    },

    /// Three-component linear colors, each float component `>= 0`.
    LinearRgb {
        /// The pooled colors.
        values: IdVec<BVoxPoolValue, [f64; 3]>,
    },

    /// Four-component linear colors, each float component `>= 0`.
    LinearRgba {
        /// The pooled colors.
        values: IdVec<BVoxPoolValue, [f64; 4]>,
    },
}

impl VoxValuePool {
    /// The number of values in the pool, across every kind. A palette's
    /// value ids into this pool must fall in `[0, values_len)`.
    pub fn values_len(&self) -> usize {
        match self {
            VoxValuePool::Json { values } => values.len(),
            VoxValuePool::Bool { values } => values.len(),
            VoxValuePool::Float { values, .. } => values.len(),
            VoxValuePool::Int { values, .. } => values.len(),
            VoxValuePool::String { values } => values.len(),
            VoxValuePool::Srgb { values } => values.len(),
            VoxValuePool::Srgba { values } => values.len(),
            VoxValuePool::LinearRgb { values } => values.len(),
            VoxValuePool::LinearRgba { values } => values.len(),
        }
    }

    /// Whether `id` is one of this pool's values: a value id in
    /// `[0, values_len)`.
    pub fn contains_value(&self, id: U32Id<BVoxPoolValue>) -> bool {
        (id.to_u32() as usize) < self.values_len()
    }

    /// Keeps only the values at `keep`, in that order, dropping the rest. Each
    /// `keep` entry is a value id below [`values_len`](Self::values_len).
    pub(crate) fn retain_values(&mut self, keep: &[U32Id<BVoxPoolValue>]) {
        fn picked<T: Clone>(
            values: &IdVec<BVoxPoolValue, T>,
            keep: &[U32Id<BVoxPoolValue>],
        ) -> IdVec<BVoxPoolValue, T> {
            IdVec::from_vec(
                keep.iter()
                    .map(|&value_id| values[value_id.to_usize_id()].clone())
                    .collect(),
            )
        }

        match self {
            VoxValuePool::Json { values } => *values = picked(values, keep),
            VoxValuePool::Bool { values } => *values = picked(values, keep),
            VoxValuePool::Float { values, .. } => *values = picked(values, keep),
            VoxValuePool::Int { values, .. } => *values = picked(values, keep),
            VoxValuePool::String { values } => *values = picked(values, keep),
            VoxValuePool::Srgb { values } => *values = picked(values, keep),
            VoxValuePool::Srgba { values } => *values = picked(values, keep),
            VoxValuePool::LinearRgb { values } => *values = picked(values, keep),
            VoxValuePool::LinearRgba { values } => *values = picked(values, keep),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{VoxBound, VoxValuePool};
    use branded_id::IdVec;

    #[test]
    fn bounded_float_pool_reports_values_len() {
        let pool = VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: IdVec::from_vec(vec![0.0, 0.5, 1.0]),
        };

        assert_eq!(pool.values_len(), 3);
    }

    #[test]
    fn color_pool_holds_typed_float_components() {
        let pool = VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[1.0, 0.0, 0.0, 1.0]]),
        };

        assert_eq!(pool.values_len(), 1);
    }
}
