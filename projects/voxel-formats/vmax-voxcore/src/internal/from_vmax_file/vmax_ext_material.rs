use crate::{VMaxExtMaterial, VMaxExtMaterialDispersion};
use vmax::VMaxMaterial;

/// The exact ext copy of a Voxel Max material.
pub(crate) fn vmax_ext_material(material: &VMaxMaterial) -> VMaxExtMaterial {
    VMaxExtMaterial {
        metallic: material.mc,
        roughness: material.rc,
        emissive: material.sic,
        shadows: material.sh,
        transmission_color: material.tc,
        dispersion: material.md.as_ref().map(|d| VMaxExtMaterialDispersion {
            absorption: d.absorption,
            ior: d.ior,
            transmission: d.transmission,
        }),
    }
}
