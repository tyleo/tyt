use crate::VMaxMaterialPalette;
use vmax::VXMaterialPalette;

/// Decodes a `VXMaterialPalette` payload into a core [`VMaxMaterialPalette`] — the
/// display name, the selectable material slots, and the RGBA color table.
pub fn decode_material_palette(palette: &VXMaterialPalette) -> VMaxMaterialPalette {
    VMaxMaterialPalette {
        name: palette.name.clone(),
        materials: palette.materials.iter().cloned().map(Into::into).collect(),
        colors: palette
            .colors
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect(),
    }
}
