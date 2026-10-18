use ty_math::TyLinSrgbF64;
use voxcore::material;
use voxrender::RenderMaterial;

/// The reflectance of `material` at normal incidence.
pub fn normal_reflectance(material: &RenderMaterial) -> TyLinSrgbF64 {
    material::normal_reflectance(material.base_color.color, material.metallic, material.ior)
}
