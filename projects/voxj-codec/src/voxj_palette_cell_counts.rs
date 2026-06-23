use voxj::VoxjPalette;

/// The cell count of each referenced palette, in `palette_refs` order, ready to
/// pass as the `cell_counts` argument of [`encode_voxj_object`](crate::encode_voxj_object)
/// and [`decode_voxj_object`](crate::decode_voxj_object). A ref that points outside
/// `palettes` contributes `0`.
pub fn voxj_palette_cell_counts(palette_refs: &[usize], palettes: &[VoxjPalette]) -> Vec<usize> {
    palette_refs
        .iter()
        .map(|&r| palettes.get(r).map_or(0, |p| p.data.len()))
        .collect()
}
