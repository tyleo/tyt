use crate::{VoxjBound, VoxjValue, VoxjValuePoolKind};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A shared pool of values, all of one shape. The variant is the pool's kind:
/// the bounded kinds carry `min`/`max` and typed numeric values, the unbounded
/// kinds carry only their values. Palettes reference pools by index and read
/// values out by value-index.
///
/// Serde tags the variant by a `kind` field, so the wire shape is
/// `{ "kind", "values" }` plus `min`/`max` on the bounded kinds. Value shapes
/// are typed per kind, so a malformed value, a missing bound on a bounded kind,
/// a stray bound on an unbounded kind, and an unknown key all reject at parse.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum VoxjValuePool {
    /// Arbitrary JSON values, including `null`.
    Json {
        /// The pooled values.
        values: Vec<VoxjValue>,
    },

    /// Boolean values.
    Bool {
        /// The pooled values.
        values: Vec<bool>,
    },

    /// Finite floating-point values within `min`/`max`.
    Float {
        /// Lower bound, a finite number or `"none"`.
        min: VoxjBound,

        /// Upper bound, a finite number or `"none"`.
        max: VoxjBound,

        /// The pooled values.
        values: Vec<f64>,
    },

    /// Integer values within `min`/`max`.
    Int {
        /// Lower bound, a finite number or `"none"`.
        min: VoxjBound,

        /// Upper bound, a finite number or `"none"`.
        max: VoxjBound,

        /// The pooled values.
        values: Vec<i64>,
    },

    /// String values.
    String {
        /// The pooled values.
        values: Vec<String>,
    },

    /// Three-component sRGB float colors, each component within `min`/`max`.
    SrgbFloat {
        /// Lower bound applied per component.
        min: VoxjBound,

        /// Upper bound applied per component.
        max: VoxjBound,

        /// The pooled colors.
        values: Vec<[f64; 3]>,
    },

    /// `#RRGGBB` sRGB hex color strings.
    SrgbHex {
        /// The pooled colors.
        values: Vec<String>,
    },

    /// Four-component sRGB float colors, each component within `min`/`max`.
    SrgbaFloat {
        /// Lower bound applied per component.
        min: VoxjBound,

        /// Upper bound applied per component.
        max: VoxjBound,

        /// The pooled colors.
        values: Vec<[f64; 4]>,
    },

    /// `#RRGGBBAA` sRGB hex color strings.
    SrgbaHex {
        /// The pooled colors.
        values: Vec<String>,
    },

    /// Three-component linear float colors, each component within `min`/`max`.
    LinearRgbFloat {
        /// Lower bound applied per component.
        min: VoxjBound,

        /// Upper bound applied per component.
        max: VoxjBound,

        /// The pooled colors.
        values: Vec<[f64; 3]>,
    },

    /// Four-component linear float colors, each component within `min`/`max`.
    LinearRgbaFloat {
        /// Lower bound applied per component.
        min: VoxjBound,

        /// Upper bound applied per component.
        max: VoxjBound,

        /// The pooled colors.
        values: Vec<[f64; 4]>,
    },
}

impl VoxjValuePool {
    /// The pool's kind tag.
    pub fn kind(&self) -> VoxjValuePoolKind {
        match self {
            VoxjValuePool::Json { .. } => VoxjValuePoolKind::Json,
            VoxjValuePool::Bool { .. } => VoxjValuePoolKind::Bool,
            VoxjValuePool::Float { .. } => VoxjValuePoolKind::Float,
            VoxjValuePool::Int { .. } => VoxjValuePoolKind::Int,
            VoxjValuePool::String { .. } => VoxjValuePoolKind::String,
            VoxjValuePool::SrgbFloat { .. } => VoxjValuePoolKind::SrgbFloat,
            VoxjValuePool::SrgbHex { .. } => VoxjValuePoolKind::SrgbHex,
            VoxjValuePool::SrgbaFloat { .. } => VoxjValuePoolKind::SrgbaFloat,
            VoxjValuePool::SrgbaHex { .. } => VoxjValuePoolKind::SrgbaHex,
            VoxjValuePool::LinearRgbFloat { .. } => VoxjValuePoolKind::LinearRgbFloat,
            VoxjValuePool::LinearRgbaFloat { .. } => VoxjValuePoolKind::LinearRgbaFloat,
        }
    }
}
