#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The closed vocabulary tagging the shape of a value pool's values. A consumer
/// must understand every kind in a file to validate it and rejects a file whose
/// `kind` it does not recognize.
///
/// The bounded kinds are `int` and `float`; they carry `min`/`max`. Colors
/// carry no bounds; each color kind's range is fixed by its color space. Colors
/// span the sRGB and linear spaces in hex and float forms; hex is sRGB only and
/// a linear color is always float.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum VoxjValuePoolKind {
    /// Any JSON value, including `null`.
    Json,

    /// A JSON boolean.
    Bool,

    /// A finite floating-point number within `min`/`max`.
    Float,

    /// An integer-valued finite number within `min`/`max`.
    Int,

    /// A JSON string.
    String,

    /// Three sRGB float components, each in `[0, 1]`.
    SrgbFloat,

    /// An `#RRGGBB` sRGB hex string.
    SrgbHex,

    /// Four sRGB float components, each in `[0, 1]`.
    SrgbaFloat,

    /// An `#RRGGBBAA` sRGB hex string.
    SrgbaHex,

    /// Three linear float components, each `>= 0`.
    LinearRgbFloat,

    /// Four linear float components, each `>= 0`.
    LinearRgbaFloat,
}
