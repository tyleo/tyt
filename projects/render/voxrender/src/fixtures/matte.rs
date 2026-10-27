use crate::RenderMaterial;
use ty_math::TyLinSrgbaF64;

/// A rough dielectric of `base_color`.
pub fn matte(base_color: TyLinSrgbaF64) -> RenderMaterial {
    RenderMaterial {
        base_color,
        metallic: 0.0,
        roughness: 1.0,
        ..RenderMaterial::default()
    }
}
