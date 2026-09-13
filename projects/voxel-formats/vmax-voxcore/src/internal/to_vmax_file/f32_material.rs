use crate::to_f32;
use vmax::{VMaxMaterial, VMaxMaterialDispersion};

/// A copy of `material` with every coefficient f32-rounded by [`to_f32`].
pub(crate) fn f32_material(material: &VMaxMaterial) -> VMaxMaterial {
    VMaxMaterial {
        mi: material.mi.clone(),
        mc: to_f32(material.mc),
        rc: to_f32(material.rc),
        sic: to_f32(material.sic),
        sh: material.sh,
        tc: material.tc.map(to_f32),
        md: material
            .md
            .as_ref()
            .map(|dispersion| VMaxMaterialDispersion {
                absorption: to_f32(dispersion.absorption),
                ior: to_f32(dispersion.ior),
                transmission: to_f32(dispersion.transmission),
            }),
    }
}
