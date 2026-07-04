use voxcore::VoxPalette;
use voxj::{VoxjPalette, VoxjPaletteBinding};

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting bindings and
/// materials in id order so each lands at its original index.
///
/// A voxcore binding's value-pool id becomes the wire `poolRef`. voxcore stores
/// materials as per-material rows of value-indices; the wire wants them
/// column-major, one column per binding, so this reads each binding's column
/// down the materials.
pub fn voxj_palette_from_vox_palette(palette: &VoxPalette) -> VoxjPalette {
    let bindings: Vec<VoxjPaletteBinding> = palette
        .iter_bindings()
        .map(|(_, binding)| VoxjPaletteBinding {
            attribute: binding.attribute.clone(),
            pool_ref: binding.pool.to_u32() as usize,
        })
        .collect();

    // Binding and material ids, reused to read each column down the materials.
    let binding_ids: Vec<_> = palette.iter_bindings().map(|(id, _)| id).collect();
    let material_ids: Vec<_> = palette.iter_materials().collect();

    let materials = binding_ids
        .iter()
        .map(|&binding_id| {
            material_ids
                .iter()
                .map(|&material_id| {
                    palette
                        .value_index(material_id, binding_id)
                        .expect("a material has a value-index for every binding")
                        as usize
                })
                .collect()
        })
        .collect();

    VoxjPalette {
        bindings,
        materials,
    }
}
