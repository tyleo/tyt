use crate::{DEFAULT_METALLIC, DEFAULT_ROUGHNESS, to_f32};
use vmax::VMaxMaterial;

/// The neutral default material Voxel Max fills the slot at `slot` with.
pub(crate) fn default_material(slot: usize) -> VMaxMaterial {
    VMaxMaterial {
        mi: (slot + 1).to_string(),
        mc: to_f32(DEFAULT_METALLIC),
        rc: to_f32(DEFAULT_ROUGHNESS),
        sic: 0.0,
        sh: true,
        tc: None,
        md: None,
    }
}
