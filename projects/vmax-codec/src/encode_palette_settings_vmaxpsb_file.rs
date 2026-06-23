use vmax::{
    VMaxCodecMaterial, VMaxCodecPaletteSettingsVmaxpsbFile, VMaxSerdeMaterial,
    VMaxSerdePaletteSettingsVmaxpsbFile,
};

/// Encodes a [`VMaxCodecPaletteSettingsVmaxpsbFile`] into a
/// `VMaxSerdePaletteSettingsVmaxpsbFile`. The inverse of
/// [`decode_palette_settings_vmaxpsb_file`](crate::decode_palette_settings_vmaxpsb_file).
pub fn encode_palette_settings_vmaxpsb_file(
    palette: &VMaxCodecPaletteSettingsVmaxpsbFile,
) -> VMaxSerdePaletteSettingsVmaxpsbFile {
    VMaxSerdePaletteSettingsVmaxpsbFile {
        name: palette.name.clone(),
        materials: palette
            .materials
            .iter()
            .enumerate()
            .map(|(slot, material)| encode_material(material, slot))
            .collect(),
        colors: palette.colors.iter().flatten().copied().collect(),
        indices: palette.indices.clone(),
        lc: palette.lc.clone(),
        palette_type: palette.palette_type,
        transparency: palette.transparency,
        r: palette.r,
        rt: palette.rt.clone(),
        cmt: palette.cmt.clone(),
        current: palette.current,
        ali: palette.ali.clone(),
    }
}

/// Re-encodes one decoded [`Material`] into a `VMaxSerdeMaterial`, setting its
/// `mi` slot token from the slot position (Voxel Max numbers material slots
/// from 1).
fn encode_material(material: &VMaxCodecMaterial, slot: usize) -> VMaxSerdeMaterial {
    VMaxSerdeMaterial {
        mi: (slot + 1).to_string(),
        mc: material.metalness,
        rc: material.roughness,
        sic: material.emission,
        sh: material.enable_shadows,
        md: material.dispersion,
    }
}
