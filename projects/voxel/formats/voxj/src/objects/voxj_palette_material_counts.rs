use crate::{
    VoxjPalette,
    objects::{Error, Result},
};

/// The material count M of each referenced palette, in `layers` order. This is
/// the `material_counts` argument that
/// [`encode_voxj_object`](crate::objects::encode_voxj_object()) and
/// [`decode_voxj_object`](crate::objects::decode_voxj_object()) need to derive the bit
/// width of `packed-base64` channels, one channel per layer. A layer entry
/// outside `palettes` is an error.
///
/// `materials` holds one row per material, so M is `materials.len()`.
pub fn voxj_palette_material_counts(
    layers: &[usize],
    palettes: &[VoxjPalette],
) -> Result<Vec<usize>> {
    layers
        .iter()
        .map(|&palette_index| {
            palettes
                .get(palette_index)
                .map(|palette| palette.materials.len())
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "layer references palette {palette_index}, but the document has {} palettes",
                        palettes.len()
                    ))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{VoxjPalette, objects::voxj_palette_material_counts, test::palette};

    #[test]
    fn maps_layers_to_referenced_material_counts() {
        let palettes = [palette(6), palette(2)];
        assert_eq!(
            voxj_palette_material_counts(&[1, 0, 1], &palettes).unwrap(),
            vec![2, 6, 2]
        );
    }

    #[test]
    fn errors_on_layer_outside_palettes() {
        let palettes = [palette(6)];
        assert!(voxj_palette_material_counts(&[0, 1], &palettes).is_err());
    }

    #[test]
    fn counts_a_property_less_palette_by_its_rows() {
        // With no properties every row is empty, but each row is still
        // one material, so M is the row count.
        let palettes = [VoxjPalette {
            properties: vec![],
            materials: vec![vec![], vec![], vec![]],
        }];
        assert_eq!(
            voxj_palette_material_counts(&[0], &palettes).unwrap(),
            vec![3]
        );
    }
}
