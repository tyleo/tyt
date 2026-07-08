use voxcore::VoxPalette;
use voxj::{VoxjPalette, VoxjPaletteBinding};

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting bindings and
/// materials in id order so each lands at its original index.
///
/// A voxcore binding's value-pool id becomes the wire `poolRef`. voxcore stores
/// materials as per-material rows of value-indices; the wire wants them
/// column-major, one column per binding, so this reads each binding's column
/// down the materials. A binding-less palette has no columns; the wire stores
/// one empty array per material instead, so the material count survives.
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

    let materials = if binding_ids.is_empty() {
        material_ids.iter().map(|_| Vec::new()).collect()
    } else {
        binding_ids
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
            .collect()
    };

    VoxjPalette {
        bindings,
        materials,
    }
}

#[cfg(test)]
mod tests {
    use crate::voxj_palette_from_vox_palette;
    use voxcore::VoxPalette;

    #[test]
    fn writes_a_binding_less_palette_as_empty_arrays() {
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        palette.add_material(vec![]).unwrap();

        let out = voxj_palette_from_vox_palette(&palette);
        assert!(out.bindings.is_empty());
        // One empty array per material, so the material count survives.
        assert_eq!(out.materials, [Vec::<usize>::new(), Vec::new()]);
    }
}
