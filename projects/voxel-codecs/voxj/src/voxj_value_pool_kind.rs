#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The closed vocabulary tagging the shape of a value pool's values. A consumer
/// must understand every kind in a file to validate it and rejects a file whose
/// `kind` it does not recognize.
///
/// The bounded kinds are `int`, `float`, and the four vector color kinds; they
/// carry `min`/`max`. Colors span the sRGB and linear spaces in hex and float
/// forms; hex is sRGB only and a linear color is always float.
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

    /// Three sRGB float components, each within `min`/`max`.
    SrgbFloat,

    /// An `#RRGGBB` sRGB hex string.
    SrgbHex,

    /// Four sRGB float components, each within `min`/`max`.
    SrgbaFloat,

    /// An `#RRGGBBAA` sRGB hex string.
    SrgbaHex,

    /// Three linear float components, each within `min`/`max`.
    LinearRgbFloat,

    /// Four linear float components, each within `min`/`max`.
    LinearRgbaFloat,
}
