use crate::{Error, Result};
use branded_id::{U32Id, ext::U32Ext};
use std::collections::HashSet;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxPalette`] from a [`VoxjPalette`], in listing order so each
/// property and material id equals its wire index.
///
/// Properties carry over as name plus pool reference, the wire `valuePool`
/// becoming a value-pool id. `materials` carries over one row per material,
/// a value-index per property.
///
/// Errors on a duplicate property name or a row whose length disagrees with
/// the properties. Pool-reference and value-id ranges are checked
/// later by [`VoxMain::validate`](voxcore::VoxMain::validate).
pub fn vox_palette_from_voxj_palette(palette: &VoxjPalette) -> Result<VoxPalette> {
    let mut out = VoxPalette::default();

    let mut names = HashSet::new();
    for property in &palette.properties {
        if !names.insert(property.name.as_str()) {
            return Err(Error::Invalid(format!(
                "palette declares property \"{}\" more than once",
                property.name
            )));
        }
        out.add_property(
            property.name.clone(),
            U32Id::from_u32(property.value_pool as u32),
        );
    }

    for (index, row) in palette.materials.iter().enumerate() {
        let value_ids = row
            .iter()
            .map(|&value_index| (value_index as u32).to_u32_id())
            .collect();
        out.add_material(value_ids).ok_or_else(|| {
            Error::Invalid(format!(
                "palette material {index} has {} value-indices but {} properties",
                row.len(),
                palette.properties.len()
            ))
        })?;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::vox_palette_from_voxj_palette;
    use branded_id::U32Id;
    use voxj::{VoxjPalette, VoxjProperty};

    fn property(name: &str, value_pool: usize) -> VoxjProperty {
        VoxjProperty {
            name: name.to_owned(),
            value_pool,
        }
    }

    #[test]
    fn maps_material_rows_one_to_one() {
        let palette = VoxjPalette {
            properties: vec![
                property("baseColorFactor", 0),
                property("metallicFactor", 1),
            ],
            materials: vec![vec![0, 2], vec![1, 0], vec![2, 1]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.property_count(), 2);
        assert_eq!(out.material_count(), 3);

        let base = out.property_by_name("baseColorFactor").unwrap();
        let metallic = out.property_by_name("metallicFactor").unwrap();
        let material_2 = out.iter_materials().nth(2).unwrap();
        // Material 2 reads value id 2 for base color and 1 for metallic.
        assert_eq!(out.value_id(material_2, base), Some(U32Id::from_u32(2)));
        assert_eq!(out.value_id(material_2, metallic), Some(U32Id::from_u32(1)));
    }

    #[test]
    fn reads_a_property_less_palette_keeping_its_material_count() {
        // With no properties every row is empty; each mints a material
        // with no value ids.
        let palette = VoxjPalette {
            properties: vec![],
            materials: vec![vec![], vec![], vec![]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.property_count(), 0);
        assert_eq!(out.material_count(), 3);
    }

    #[test]
    fn rejects_a_non_empty_row_without_properties() {
        let palette = VoxjPalette {
            properties: vec![],
            materials: vec![vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_duplicate_property_name() {
        let palette = VoxjPalette {
            properties: vec![property("rgba", 0), property("rgba", 1)],
            materials: vec![vec![0, 0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_short_material_row() {
        let palette = VoxjPalette {
            properties: vec![property("a", 0), property("b", 1)],
            materials: vec![vec![0, 1], vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_long_material_row() {
        let palette = VoxjPalette {
            properties: vec![property("a", 0), property("b", 1)],
            materials: vec![vec![0, 1], vec![0, 1, 2]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }
}
