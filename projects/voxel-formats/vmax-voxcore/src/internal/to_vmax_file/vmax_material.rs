use crate::VMaxExtMaterial;
use vmax::{VMaxMaterial, VMaxMaterialDispersion};

/// Rebuilds a Voxel Max material from its exact ext copy. The `mi` token is
/// derived from the 1-based slot and the transparency color `tc` is dropped,
/// matching the writer's behavior.
pub(crate) fn vmax_material(slot: usize, material: &VMaxExtMaterial) -> VMaxMaterial {
    VMaxMaterial {
        mi: (slot + 1).to_string(),
        mc: material.metallic,
        rc: material.roughness,
        sic: material.emissive,
        sh: material.shadows,
        tc: material.transmission_color,
        md: material
            .dispersion
            .as_ref()
            .map(|dispersion| VMaxMaterialDispersion {
                absorption: dispersion.absorption,
                ior: dispersion.ior,
                transmission: dispersion.transmission,
            }),
    }
}
