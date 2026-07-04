use crate::{VoxBound, VoxValue, VoxValuePoolKind};

/// A shared pool of values, all of one shape, that palettes reference by id and
/// read out by value-index. The variant is the pool's
/// [`VoxValuePoolKind`](crate::VoxValuePoolKind): the bounded kinds (`float`,
/// `int`) carry `min`/`max` and typed numeric values; every other kind carries
/// only its typed values.
///
/// Colors are stored as float components in the color space's natural range,
/// one canonical kind per space and alpha combination, so the wire format's
/// separate hex and float color encodings both map onto the matching variant
/// here. Only the `json` kind holds a [`VoxValue`](crate::VoxValue).
/// [`VoxMain::validate`](crate::VoxMain::validate) checks each value against its
/// kind and the bounds.
#[derive(Clone, Debug, PartialEq)]
pub enum VoxValuePool {
    /// Arbitrary [`VoxValue`](crate::VoxValue)s, including null.
    Json {
        /// The pooled values.
        values: Vec<VoxValue>,
    },

    /// Boolean values.
    Bool {
        /// The pooled values.
        values: Vec<bool>,
    },

    /// Finite floating-point values within `min`/`max`.
    Float {
        /// Lower bound, a finite number or unbounded.
        min: VoxBound,

        /// Upper bound, a finite number or unbounded.
        max: VoxBound,

        /// The pooled values.
        values: Vec<f64>,
    },

    /// Integer values within `min`/`max`.
    Int {
        /// Lower bound, a finite number or unbounded.
        min: VoxBound,

        /// Upper bound, a finite number or unbounded.
        max: VoxBound,

        /// The pooled values.
        values: Vec<i64>,
    },

    /// String values.
    String {
        /// The pooled values.
        values: Vec<String>,
    },

    /// Three-component sRGB colors, each float component in `[0, 1]`.
    Srgb {
        /// The pooled colors.
        values: Vec<[f64; 3]>,
    },

    /// Four-component sRGB colors, each float component in `[0, 1]`.
    Srgba {
        /// The pooled colors.
        values: Vec<[f64; 4]>,
    },

    /// Three-component linear colors, each float component `>= 0`.
    LinearRgb {
        /// The pooled colors.
        values: Vec<[f64; 3]>,
    },

    /// Four-component linear colors, each float component `>= 0`.
    LinearRgba {
        /// The pooled colors.
        values: Vec<[f64; 4]>,
    },
}

impl VoxValuePool {
    /// The number of values in the pool, across every kind. A palette's
    /// value-indices into this pool must fall in `[0, values_len)`.
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

    /// The pool's kind.
    pub fn kind(&self) -> VoxValuePoolKind {
        match self {
            VoxValuePool::Json { .. } => VoxValuePoolKind::Json,
            VoxValuePool::Bool { .. } => VoxValuePoolKind::Bool,
            VoxValuePool::Float { .. } => VoxValuePoolKind::Float,
            VoxValuePool::Int { .. } => VoxValuePoolKind::Int,
            VoxValuePool::String { .. } => VoxValuePoolKind::String,
            VoxValuePool::Srgb { .. } => VoxValuePoolKind::Srgb,
            VoxValuePool::Srgba { .. } => VoxValuePoolKind::Srgba,
            VoxValuePool::LinearRgb { .. } => VoxValuePoolKind::LinearRgb,
            VoxValuePool::LinearRgba { .. } => VoxValuePoolKind::LinearRgba,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{VoxBound, VoxValuePool, VoxValuePoolKind};

    #[test]
    fn bounded_float_pool_reports_kind_and_values_len() {
        let pool = VoxValuePool::Float {
            min: VoxBound::Number(0.0),
            max: VoxBound::None,
            values: vec![0.0, 0.5, 1.0],
        };

        assert_eq!(pool.kind(), VoxValuePoolKind::Float);
        assert_eq!(pool.values_len(), 3);
    }

    #[test]
    fn color_pool_holds_typed_float_components() {
        let pool = VoxValuePool::Srgba {
            values: vec![[1.0, 0.0, 0.0, 1.0]],
        };

        assert_eq!(pool.kind(), VoxValuePoolKind::Srgba);
        assert_eq!(pool.values_len(), 1);
    }
}
