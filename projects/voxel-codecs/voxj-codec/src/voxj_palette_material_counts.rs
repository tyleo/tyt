use crate::{Error, Result};
use voxj::VoxjPalette;

/// The material count M of each referenced palette, in `layers` order. This is
/// the `material_counts` argument that
/// [`encode_voxj_object`](crate::encode_voxj_object()) and
/// [`decode_voxj_object`](crate::decode_voxj_object()) need to derive the bit
/// width of `packed-base64` channels, one channel per layer. A layer entry
/// outside `palettes` is an error.
///
/// `materials` is row-major, one row per material, so M is `materials.len()`.
pub fn voxj_palette_material_counts(
    layers: &[usize],
    palettes: &[VoxjPalette],
) -> Result<Vec<usize>> {
    layers
        .iter()
        .map(|&idx| {
            palettes
                .get(idx)
                .map(|palette| palette.materials.len())
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "layer references palette {idx}, but the document has {} palettes",
                        palettes.len()
                    ))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::voxj_palette_material_counts;
    use voxj::{VoxjArrayProperty, VoxjPalette};

    /// A palette of `m` materials: one array property over pool 0, its rows
    /// the value-indices `0..m`.
    fn palette(m: usize) -> VoxjPalette {
        VoxjPalette {
            array_properties: vec![VoxjArrayProperty {
                name: "baseColorFactor".to_owned(),
                value_pool: 0,
            }],
            materials: (0..m).map(|i| vec![i]).collect(),
        }
    }

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
        // With no array properties every row is empty, but each row is still
        // one material, so M is the row count.
        let palettes = [VoxjPalette {
            array_properties: vec![],
            materials: vec![vec![], vec![], vec![]],
        }];
        assert_eq!(
            voxj_palette_material_counts(&[0], &palettes).unwrap(),
            vec![3]
        );
    }
}
