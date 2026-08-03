use ty_math::{TyLinSrgbaF64, TySrgbaU8};

/// Decodes an 8-bit sRGB color to linear light, the import-side half of the
/// one transfer voxsmith applies at its boundaries. Each color component
/// normalizes to `[0, 1]` and decodes through the sRGB transfer. The alpha
/// carries no gamma, so it only normalizes.
pub(crate) fn lin_srgba_f64_from_srgba_u8(color: TySrgbaU8) -> TyLinSrgbaF64 {
    color.into_format::<f64, f64>().into_linear()
}
