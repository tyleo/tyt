use crate::MaterialPalette;
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Decodes a `VMaxPaletteSettingsVmaxpsbFile` payload into a core
/// [`MaterialPalette`] — the display name, the selectable material slots, and
/// the RGBA color table.
pub fn decode_material_palette(palette: &VMaxPaletteSettingsVmaxpsbFile) -> MaterialPalette {
    MaterialPalette {
        name: palette.name.clone(),
        materials: palette.materials.iter().cloned().map(Into::into).collect(),
        colors: palette
            .colors
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect(),
    }
}
