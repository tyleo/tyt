use crate::material_pass;
use ty_math::TyLinSrgbF64;
use voxrender::RenderMaterial;

/// Whether `material` passes no light at normal incidence.
pub fn is_opaque(material: &RenderMaterial) -> bool {
    material_pass(material, 1.0) == TyLinSrgbF64::new(0.0, 0.0, 0.0)
}
