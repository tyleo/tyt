use crate::pad_materials;
use vmax::{VMaxMaterial, VMaxPaletteSettingsVmaxpsbFile};

/// Builds a settings sidecar carrying `colors`, `materials`, and the per-color
/// material map (`lc`/`indices`/`current`). The materials are padded to the
/// fixed slot count and every coefficient f32-rounded, since Voxel Max drops a
/// palette whose material coefficients are not f32-representable. The remaining
/// editor-state keys are filled with the defaults Voxel Max expects.
pub(crate) fn material_settings(
    name: String,
    materials: Vec<VMaxMaterial>,
    colors: Vec<[u8; 4]>,
    lc: Vec<u8>,
    indices: Vec<i64>,
    current: i64,
) -> VMaxPaletteSettingsVmaxpsbFile {
    VMaxPaletteSettingsVmaxpsbFile {
        name,
        materials: pad_materials(materials),
        colors: colors.iter().flatten().copied().collect(),
        indices,
        lc,
        palette_type: 0,
        transparency: 1.0,
        r: 0,
        rt: "n".to_owned(),
        cmt: "ng".to_owned(),
        current,
        ali: "1".to_owned(),
        voxmats: Vec::new(),
        ls: Vec::new(),
    }
}
