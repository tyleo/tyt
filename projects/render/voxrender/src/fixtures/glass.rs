use crate::{RenderMaterial, fixtures::matte};
use ty_math::TyLinSrgbaF64;

/// A smooth dielectric of `base_color` at full transmission.
pub fn glass(base_color: TyLinSrgbaF64) -> RenderMaterial {
    RenderMaterial {
        roughness: 0.2,
        transmission: 1.0,
        ..matte(base_color)
    }
}
