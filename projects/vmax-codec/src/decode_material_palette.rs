use crate::MaterialPalette;
use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Decodes a `VMaxPaletteSettingsVmaxpsbFile` payload into a core
/// [`MaterialPalette`]: the display name, the selectable material slots, and
/// the RGBA color table. Errors when the packed color table's length is not a
/// multiple of 4, since each `[r, g, b, a]` entry is exactly 4 bytes.
pub fn decode_material_palette(
    palette: &VMaxPaletteSettingsVmaxpsbFile,
) -> Result<MaterialPalette> {
    if !palette.colors.len().is_multiple_of(4) {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            format!(
                "color table length {} is not a multiple of 4 (one RGBA entry is 4 bytes)",
                palette.colors.len()
            ),
        ));
    }
    Ok(MaterialPalette {
        name: palette.name.clone(),
        materials: palette.materials.iter().cloned().map(Into::into).collect(),
        colors: palette
            .colors
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect(),
    })
}
