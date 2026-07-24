use voxcore::VoxPalette;
use voxj::{VoxjArrayProperty, VoxjPalette};

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting properties and
/// materials in id order so each lands at its original index.
///
/// A voxcore property's value-pool id becomes the wire `valuePool`.
/// `materials` is row-major on both sides, one row per material with a
/// value-index per array property, so rows map one to one.
pub fn voxj_palette_from_vox_palette(palette: &VoxPalette) -> VoxjPalette {
    let array_properties: Vec<VoxjArrayProperty> = palette
        .iter_array_properties()
        .map(|(_, property)| VoxjArrayProperty {
            name: property.name.clone(),
            value_pool: property.pool.to_u32() as usize,
        })
        .collect();

    // Array property ids, reused to read each material's row in property
    // order.
    let array_property_ids: Vec<_> = palette.iter_array_properties().map(|(id, _)| id).collect();

    let materials = palette
        .iter_materials()
        .map(|material_id| {
            array_property_ids
                .iter()
                .map(|&array_property_id| {
                    palette
                        .value_id(material_id, array_property_id)
                        .expect("a material has a value id for every array property")
                        .to_u32() as usize
                })
                .collect()
        })
        .collect();

    VoxjPalette {
        array_properties,
        materials,
    }
}

#[cfg(test)]
mod tests {
    use crate::voxj_palette_from_vox_palette;
    use voxcore::VoxPalette;

    #[test]
    fn writes_a_property_less_palette_as_empty_rows() {
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        palette.add_material(vec![]).unwrap();

        let out = voxj_palette_from_vox_palette(&palette);
        assert!(out.array_properties.is_empty());
        // One empty row per material, so the material count survives.
        assert_eq!(out.materials, [Vec::<usize>::new(), Vec::new()]);
    }
}
