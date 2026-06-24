use crate::{Error, Result};
use voxj::VoxjPalette;

/// The cell count of each referenced palette, in `palette_refs` order, ready to
/// pass as the `cell_counts` argument of [`encode_voxj_object`](crate::encode_voxj_object)
/// and [`decode_voxj_object`](crate::decode_voxj_object). A ref that points outside
/// `palettes` is an error: the object would sample a palette that does not exist.
pub fn voxj_palette_cell_counts(
    palette_refs: &[usize],
    palettes: &[VoxjPalette],
) -> Result<Vec<usize>> {
    palette_refs
        .iter()
        .map(|&idx| {
            palettes
                .get(idx)
                .map(|palette| palette.data.len())
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "palette ref {idx} is out of bounds: the document has {} palettes",
                        palettes.len()
                    ))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::voxj_palette_cell_counts;
    use voxj::VoxjPalette;

    fn palette(n: usize) -> VoxjPalette {
        VoxjPalette {
            attributes: vec!["rgba".to_owned()],
            data: (0..n).map(|_| Vec::new()).collect(),
        }
    }

    #[test]
    fn maps_refs_to_referenced_cell_counts() {
        let palettes = [palette(6), palette(2)];
        assert_eq!(
            voxj_palette_cell_counts(&[1, 0, 1], &palettes).unwrap(),
            vec![2, 6, 2]
        );
    }

    #[test]
    fn errors_on_ref_outside_palettes() {
        let palettes = [palette(6)];
        assert!(voxj_palette_cell_counts(&[0, 1], &palettes).is_err());
    }
}
