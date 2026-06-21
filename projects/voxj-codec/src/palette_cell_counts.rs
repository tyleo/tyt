use voxj::{VoxjObject, VoxjPalette};

/// The cell count of each palette an object references, in
/// [`palette_refs`](VoxjObject::palette_refs) order, ready to pass as
/// [`decode_object`](crate::decode_object)'s `cell_counts` argument. A
/// `palette_ref` that points outside `palettes` contributes `0`.
pub fn palette_cell_counts(object: &VoxjObject, palettes: &[VoxjPalette]) -> Vec<usize> {
    object
        .palette_refs
        .iter()
        .map(|&r| palettes.get(r).map_or(0, |p| p.data.len()))
        .collect()
}
