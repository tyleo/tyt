use voxcore::VoxPalette;
use voxj::{VoxjPalette, VoxjProperty};

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting properties and
/// materials in id order so each lands at its original index.
///
/// A voxcore property's value-pool id becomes the wire `valuePool`.
/// `materials` carries over one row per material, a value-index per
/// property.
pub fn voxj_palette_from_vox_palette(palette: &VoxPalette) -> VoxjPalette {
    let properties: Vec<VoxjProperty> = palette
        .iter_properties()
        .map(|(_, property)| VoxjProperty {
            name: property.name.clone(),
            value_pool: property.value_pool_id.to_u32() as usize,
        })
        .collect();

    // Property ids, reused to read each material's row in property
    // order.
    let property_ids: Vec<_> = palette
        .iter_properties()
        .map(|(property_id, _)| property_id)
        .collect();

    let materials = palette
        .iter_materials()
        .map(|material_id| {
            property_ids
                .iter()
                .map(|&property_id| {
                    palette
                        .value_id(material_id, property_id)
                        .expect("a material has a value id for every property")
                        .to_u32() as usize
                })
                .collect()
        })
        .collect();

    VoxjPalette {
        properties,
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
        palette.retain_material(vec![]).unwrap();
        palette.retain_material(vec![]).unwrap();

        let out = voxj_palette_from_vox_palette(&palette);
        assert!(out.properties.is_empty());
        // One empty row per material, so the material count survives.
        assert_eq!(out.materials, [Vec::<usize>::new(), Vec::new()]);
    }
}
