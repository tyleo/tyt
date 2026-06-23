use std::io::{Error as IOError, ErrorKind, Result};
use vmax::{
    VMaxCodecMaterial, VMaxCodecPaletteSettingsVmaxpsbFile, VMaxSerdePaletteSettingsVmaxpsbFile,
};

/// Decodes a `VMaxSerdePaletteSettingsVmaxpsbFile` into a [`VMaxCodecPaletteSettingsVmaxpsbFile`]:
/// decodes each material slot and unpacks the RGBA color table, carrying the
/// palette-level editor state through unchanged. The inverse of
/// [`encode_palette_settings_vmaxpsb_file`](crate::encode_palette_settings_vmaxpsb_file). Errors when the
/// packed color table's length is not a multiple of 4, since each `[r, g, b, a]`
/// entry is exactly 4 bytes.
pub fn decode_palette_settings_vmaxpsb_file(
    palette: &VMaxSerdePaletteSettingsVmaxpsbFile,
) -> Result<VMaxCodecPaletteSettingsVmaxpsbFile> {
    if !palette.colors.len().is_multiple_of(4) {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            format!(
                "color table length {} is not a multiple of 4 (one RGBA entry is 4 bytes)",
                palette.colors.len()
            ),
        ));
    }
    Ok(VMaxCodecPaletteSettingsVmaxpsbFile {
        name: palette.name.clone(),
        materials: palette
            .materials
            .iter()
            .cloned()
            .map(VMaxCodecMaterial::from)
            .collect(),
        colors: palette
            .colors
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect(),
        indices: palette.indices.clone(),
        lc: palette.lc.clone(),
        palette_type: palette.palette_type,
        transparency: palette.transparency,
        r: palette.r,
        rt: palette.rt.clone(),
        cmt: palette.cmt.clone(),
        current: palette.current,
        ali: palette.ali.clone(),
    })
}
