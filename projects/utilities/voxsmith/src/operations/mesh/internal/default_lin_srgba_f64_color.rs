use ty_math::TyLinSrgbaF64;
use voxcore::material::{BASE_COLOR, EMISSIVE_COLOR};

/// The glTF spec default for a recommended color property, the linear factor
/// the glTF schema states, or `None` for a key with no standard default, such
/// as a custom property. A three-component color takes opaque alpha.
pub(crate) fn default_lin_srgba_f64_color(key: &str) -> Option<TyLinSrgbaF64> {
    match key {
        BASE_COLOR => Some(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0)),
        EMISSIVE_COLOR => Some(TyLinSrgbaF64::new(0.0, 0.0, 0.0, 1.0)),
        _ => None,
    }
}
