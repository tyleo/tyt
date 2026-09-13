use crate::{MATERIAL_SLOTS, default_material, f32_material};
use vmax::VMaxMaterial;

/// F32-rounds every material's coefficients and, for a palette that carries
/// materials, pads the list to [`MATERIAL_SLOTS`] with the neutral default. A
/// color-only palette keeps an empty list, since Voxel Max then uses its own
/// default materials.
pub(crate) fn pad_materials(materials: Vec<VMaxMaterial>) -> Vec<VMaxMaterial> {
    if materials.is_empty() {
        return materials;
    }
    let mut out: Vec<VMaxMaterial> = materials.iter().map(f32_material).collect();
    while out.len() < MATERIAL_SLOTS {
        out.push(default_material(out.len()));
    }
    out
}
