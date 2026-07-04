/// The shape tagging a [`VoxValuePool`](crate::VoxValuePool)'s values.
///
/// A closed vocabulary of nine kinds: five scalar or structured kinds and four
/// color kinds. The color kinds span the sRGB and linear color spaces with and
/// without alpha. A color is always stored as float components in the space's
/// natural range, so the wire format's separate hex and float color encodings
/// both canonicalize onto one of these four kinds. `int` and `float` are the
/// bounded kinds and carry `min`/`max`; every color kind is unbounded, its
/// range fixed by its color space.
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
