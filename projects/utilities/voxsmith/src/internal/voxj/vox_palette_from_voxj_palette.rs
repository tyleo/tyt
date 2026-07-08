use crate::{Error, Result};
use branded_id::U32Id;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxPalette`] from a [`VoxjPalette`], in listing order so each
/// binding and material id equals its wire index.
///
/// Bindings carry over as attribute-plus-pool-reference, the wire `poolRef`
/// becoming a value-pool id. The wire stores materials column-major, one column
/// per binding; this transposes them into voxcore's per-material rows of
/// value-indices, one index per binding. A binding-less palette instead stores
/// one empty array per material, each minting one material with no
/// value-indices.
///
/// Errors on a duplicate attribute, a `materials` column count that disagrees
/// with the bindings, a ragged column, or a non-empty material entry in a
/// binding-less palette. Pool-reference and value-index ranges are checked
/// later by [`VoxMain::validate`](voxcore::VoxMain::validate).
pub fn vox_palette_from_voxj_palette(palette: &VoxjPalette) -> Result<VoxPalette> {
    let mut out = VoxPalette::default();

    for binding in &palette.bindings {
        if out.binding_by_attribute(&binding.attribute).is_some() {
            return Err(Error::invalid(format!(
                "palette binds attribute \"{}\" more than once",
                binding.attribute
            )));
        }
        out.add_binding(
            binding.attribute.clone(),
            U32Id::from_u32(binding.pool_ref as u32),
        );
    }

    // A binding-less palette has no columns to transpose; each entry is an
    // empty array minting one material.
    if palette.bindings.is_empty() {
        for (index, entry) in palette.materials.iter().enumerate() {
            if !entry.is_empty() {
                return Err(Error::invalid(format!(
                    "palette has no bindings, so material {index} must be an empty array, \
                     not {} value-indices",
                    entry.len()
                )));
            }
            out.add_material(vec![])
                .expect("one value-index per binding");
        }
        return Ok(out);
    }

    if palette.materials.len() != palette.bindings.len() {
        return Err(Error::invalid(format!(
            "palette has {} material columns but {} bindings",
            palette.materials.len(),
            palette.bindings.len()
        )));
    }

    // Column length M, the material count; reject any column that disagrees,
    // whether shorter or longer, so a ragged palette errors rather than
    // silently dropping the surplus of a longer column.
    let material_count = palette.materials.first().map_or(0, Vec::len);
    if let Some(column) = palette
        .materials
        .iter()
        .find(|column| column.len() != material_count)
    {
        return Err(Error::invalid(format!(
            "palette has a material column of length {} but the material count is {material_count}",
            column.len()
        )));
    }

    for m in 0..material_count {
        // Every column shares length M, so `column[m]` is in bounds and the row
        // has one value-index per binding, satisfying `add_material`'s arity.
        let value_indices = palette
            .materials
            .iter()
            .map(|column| column[m] as u32)
            .collect();
        out.add_material(value_indices)
            .expect("one value-index per binding");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::vox_palette_from_voxj_palette;
    use voxj::{VoxjPalette, VoxjPaletteBinding};

    fn binding(attribute: &str, pool_ref: usize) -> VoxjPaletteBinding {
        VoxjPaletteBinding {
            attribute: attribute.to_owned(),
            pool_ref,
        }
    }

    #[test]
    fn transposes_column_major_materials_into_rows() {
        let palette = VoxjPalette {
            bindings: vec![binding("baseColorFactor", 0), binding("metallicFactor", 1)],
            materials: vec![vec![0, 1, 2], vec![2, 0, 1]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.binding_count(), 2);
        assert_eq!(out.material_count(), 3);

        let base = out.binding_by_attribute("baseColorFactor").unwrap();
        let metallic = out.binding_by_attribute("metallicFactor").unwrap();
        let material_2 = out.iter_materials().nth(2).unwrap();
        // Material 2 reads value-index 2 for base color and 1 for metallic.
        assert_eq!(out.value_index(material_2, base), Some(2));
        assert_eq!(out.value_index(material_2, metallic), Some(1));
    }

    #[test]
    fn reads_a_binding_less_palette_keeping_its_material_count() {
        // One empty array per material; each mints a material with no
        // value-indices.
        let palette = VoxjPalette {
            bindings: vec![],
            materials: vec![vec![], vec![], vec![]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.binding_count(), 0);
        assert_eq!(out.material_count(), 3);
    }

    #[test]
    fn rejects_a_non_empty_material_entry_without_bindings() {
        let palette = VoxjPalette {
            bindings: vec![],
            materials: vec![vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_duplicate_attribute() {
        let palette = VoxjPalette {
            bindings: vec![binding("rgba", 0), binding("rgba", 1)],
            materials: vec![vec![0], vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_material_column_count_mismatch() {
        let palette = VoxjPalette {
            bindings: vec![binding("a", 0), binding("b", 1)],
            materials: vec![vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_ragged_material_columns_shorter_non_first() {
        let palette = VoxjPalette {
            bindings: vec![binding("a", 0), binding("b", 1)],
            materials: vec![vec![0, 1], vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_ragged_material_columns_longer_non_first() {
        // The first column sets the material count; a longer later column must
        // still be rejected rather than silently truncated.
        let palette = VoxjPalette {
            bindings: vec![binding("a", 0), binding("b", 1)],
            materials: vec![vec![0], vec![0, 1]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }
}
