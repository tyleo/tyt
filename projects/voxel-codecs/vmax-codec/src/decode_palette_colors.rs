use crate::{Error, Result};
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Unpacks a palette settings file's embedded `colors` table into one
/// `[r, g, b, a]` cell per entry. Voxel Max stores this packed table only when a
/// palette ships no sibling `palette*.png`; an empty table yields no cells.
/// Errors when the packed length is not a multiple of 4 (one RGBA entry is 4
/// bytes).
pub fn decode_palette_colors(palette: &VMaxPaletteSettingsVmaxpsbFile) -> Result<Vec<[u8; 4]>> {
    if !palette.colors.len().is_multiple_of(4) {
        return Err(Error::Invalid(format!(
            "color table length {} is not a multiple of 4 (one RGBA entry is 4 bytes)",
            palette.colors.len()
        )));
    }
    Ok(palette
        .colors
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect())
}
