/// The shape tagging a [`VoxValuePool`](crate::VoxValuePool)'s values.
///
/// A closed vocabulary of value kinds: five scalar or structured kinds and four
/// color kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxValuePoolKind {
    /// Any [`VoxValue`](crate::VoxValue), including null.
    Json,

    /// A boolean.
    Bool,

    /// A finite floating-point number within the pool's `min`/`max`.
    Float,

    /// An integer-valued finite number within the pool's `min`/`max`.
    Int,

    /// A string.
    String,

    /// Three sRGB float components, each in `[0, 1]`.
    Srgb,

    /// Four sRGB float components, each in `[0, 1]`.
    Srgba,

    /// Three linear float components, each `>= 0`.
    LinearRgb,

    /// Four linear float components, each `>= 0`.
    LinearRgba,
}
