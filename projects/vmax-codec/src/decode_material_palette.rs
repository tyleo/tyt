use crate::VMaxMaterialPalette;
use vmax::VXMaterialPalette;

/// Decodes a `VXMaterialPalette` payload into a core [`VMaxMaterialPalette`] — the
/// display name plus the selectable material slots.
pub fn decode_material_palette(palette: &VXMaterialPalette) -> VMaxMaterialPalette {
    VMaxMaterialPalette {
        name: palette.name.clone(),
        materials: palette.materials.iter().cloned().map(Into::into).collect(),
    }
}
