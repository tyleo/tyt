use crate::{Error, Result};
use voxj::VoxjPalette;

/// The material count M of each referenced palette, in `layer_palette_refs`
/// order. This is the `material_counts` argument that
/// [`encode_voxj_object`](crate::encode_voxj_object()) and
/// [`decode_voxj_object`](crate::decode_voxj_object()) need to derive the bit
/// width of `packed-base64` samples. A ref outside `palettes` is an error.
///
/// M is the length of the palette's first materials column. Validation
/// guarantees a palette's bindings are non-empty and every column shares the
/// same length M at least 1, so the first column is authoritative; a palette
/// with no materials columns yields M = 0.
pub fn voxj_palette_material_counts(
    layer_palette_refs: &[usize],
    palettes: &[VoxjPalette],
) -> Result<Vec<usize>> {
    layer_palette_refs
        .iter()
        .map(|&idx| {
            palettes.get(idx).map(material_count).ok_or_else(|| {
                Error::Invalid(format!(
                    "palette ref {idx} is out of bounds: the document has {} palettes",
                    palettes.len()
                ))
            })
        })
        .collect()
}

/// The material count M of a palette: the length of its first materials column,
/// or 0 when it has none.
fn material_count(palette: &VoxjPalette) -> usize {
    palette.materials.first().map_or(0, Vec::len)
}

#[cfg(test)]
mod tests {
    use crate::voxj_palette_material_counts;
    use voxj::{VoxjPalette, VoxjPaletteBinding};

    /// A palette of M materials: one binding over one column of M value-indices.
    fn palette(m: usize) -> VoxjPalette {
        VoxjPalette {
            bindings: vec![VoxjPaletteBinding {
                attribute: "baseColorFactor".to_owned(),
                pool_ref: 0,
            }],
            materials: vec![(0..m).collect()],
        }
    }

    #[test]
    fn maps_refs_to_referenced_material_counts() {
        let palettes = [palette(6), palette(2)];
        assert_eq!(
            voxj_palette_material_counts(&[1, 0, 1], &palettes).unwrap(),
            vec![2, 6, 2]
        );
    }

    #[test]
    fn errors_on_ref_outside_palettes() {
        let palettes = [palette(6)];
        assert!(voxj_palette_material_counts(&[0, 1], &palettes).is_err());
    }
}
