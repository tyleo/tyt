use crate::{Error, Result};
use branded_id::{U32Id, ext::U32Ext};
use std::collections::HashSet;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxPalette`] from a [`VoxjPalette`], in listing order so each
/// property and material id equals its wire index.
///
/// Array and scalar properties carry over as name plus pool reference, the
/// wire `valuePool` becoming a value-pool id and a scalar property's
/// `valueIndex` a value id. `materials` is row-major on both sides, one row
/// per material with a value-index per array property, so rows map one to one.
///
/// Errors on a duplicate property name across the two lists or a row whose
/// length disagrees with the array properties. Pool-reference and value-id
/// ranges are checked later by
/// [`VoxMain::validate`](voxcore::VoxMain::validate).
pub fn vox_palette_from_voxj_palette(palette: &VoxjPalette) -> Result<VoxPalette> {
    let mut out = VoxPalette::default();

    let mut names = HashSet::new();
    for property in &palette.array_properties {
        if !names.insert(property.name.as_str()) {
            return Err(duplicate_name(&property.name));
        }
        out.add_array_property(
            property.name.clone(),
            U32Id::from_u32(property.value_pool as u32),
        );
    }
    for property in &palette.scalar_properties {
        if !names.insert(property.name.as_str()) {
            return Err(duplicate_name(&property.name));
        }
        out.add_scalar_property(
            property.name.clone(),
            U32Id::from_u32(property.value_pool as u32),
            (property.value_index as u32).to_u32_id(),
        );
    }

    for (index, row) in palette.materials.iter().enumerate() {
        let value_ids = row
            .iter()
            .map(|&value_index| (value_index as u32).to_u32_id())
            .collect();
        out.add_material(value_ids).ok_or_else(|| {
            Error::Invalid(format!(
                "palette material {index} has {} value-indices but {} array properties",
                row.len(),
                palette.array_properties.len()
            ))
        })?;
    }

    Ok(out)
}

/// Invalid-data error for a property name declared twice in one palette.
fn duplicate_name(name: &str) -> Error {
    Error::Invalid(format!(
        "palette declares property \"{name}\" more than once"
    ))
}

#[cfg(test)]
mod tests {
    use crate::vox_palette_from_voxj_palette;
    use branded_id::U32Id;
    use voxj::{VoxjArrayProperty, VoxjPalette, VoxjScalarProperty};

    fn array_property(name: &str, value_pool: usize) -> VoxjArrayProperty {
        VoxjArrayProperty {
            name: name.to_owned(),
            value_pool,
        }
    }

    fn scalar_property(name: &str, value_pool: usize, value_index: usize) -> VoxjScalarProperty {
        VoxjScalarProperty {
            name: name.to_owned(),
            value_pool,
            value_index,
        }
    }

    #[test]
    fn maps_material_rows_one_to_one() {
        let palette = VoxjPalette {
            array_properties: vec![
                array_property("baseColorFactor", 0),
                array_property("metallicFactor", 1),
            ],
            scalar_properties: vec![],
            materials: vec![vec![0, 2], vec![1, 0], vec![2, 1]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.array_property_count(), 2);
        assert_eq!(out.material_count(), 3);

        let base = out.array_property_by_name("baseColorFactor").unwrap();
        let metallic = out.array_property_by_name("metallicFactor").unwrap();
        let material_2 = out.iter_materials().nth(2).unwrap();
        // Material 2 reads value id 2 for base color and 1 for metallic.
        assert_eq!(out.value_id(material_2, base), Some(U32Id::from_u32(2)));
        assert_eq!(out.value_id(material_2, metallic), Some(U32Id::from_u32(1)));
    }

    #[test]
    fn carries_scalar_properties_over() {
        let palette = VoxjPalette {
            array_properties: vec![array_property("baseColorFactor", 0)],
            scalar_properties: vec![scalar_property("emissiveStrength", 1, 2)],
            materials: vec![vec![0]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.scalar_property_count(), 1);

        let strength = out.scalar_property_by_name("emissiveStrength").unwrap();
        let property = out.scalar_property(strength).unwrap();
        assert_eq!(property.pool, U32Id::from_u32(1));
        assert_eq!(property.value_id, U32Id::from_u32(2));
    }

    #[test]
    fn reads_a_property_less_palette_keeping_its_material_count() {
        // With no array properties every row is empty; each mints a material
        // with no value ids.
        let palette = VoxjPalette {
            array_properties: vec![],
            scalar_properties: vec![],
            materials: vec![vec![], vec![], vec![]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.array_property_count(), 0);
        assert_eq!(out.material_count(), 3);
    }

    #[test]
    fn reads_a_scalar_only_palette_with_no_materials() {
        let palette = VoxjPalette {
            array_properties: vec![],
            scalar_properties: vec![scalar_property("emissiveStrength", 0, 0)],
            materials: vec![],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.scalar_property_count(), 1);
        assert_eq!(out.material_count(), 0);
    }

    #[test]
    fn rejects_a_non_empty_row_without_array_properties() {
        let palette = VoxjPalette {
            array_properties: vec![],
            scalar_properties: vec![],
            materials: vec![vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_duplicate_array_property_name() {
        let palette = VoxjPalette {
            array_properties: vec![array_property("rgba", 0), array_property("rgba", 1)],
            scalar_properties: vec![],
            materials: vec![vec![0, 0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_duplicate_name_across_the_two_lists() {
        let palette = VoxjPalette {
            array_properties: vec![array_property("emissiveStrength", 0)],
            scalar_properties: vec![scalar_property("emissiveStrength", 0, 0)],
            materials: vec![vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_short_material_row() {
        let palette = VoxjPalette {
            array_properties: vec![array_property("a", 0), array_property("b", 1)],
            scalar_properties: vec![],
            materials: vec![vec![0, 1], vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_long_material_row() {
        let palette = VoxjPalette {
            array_properties: vec![array_property("a", 0), array_property("b", 1)],
            scalar_properties: vec![],
            materials: vec![vec![0, 1], vec![0, 1, 2]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }
}
