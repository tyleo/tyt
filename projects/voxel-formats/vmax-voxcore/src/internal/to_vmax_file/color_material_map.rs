use crate::PalettePlan;
use std::collections::BTreeSet;

/// The per-color material map for a palette's settings sidecar. For each color
/// cell a sample draws, sets its slot's bit in `lc` (`1 << material_idx`),
/// lists the used cells in `indices`, and takes the first as `current`. Empty
/// for a palette with no materials, which Voxel Max renders with its own
/// defaults. Voxel Max reads a voxel's material from this map, not the
/// per-voxel byte.
pub(crate) fn color_material_map(plan: &PalettePlan) -> (Vec<u8>, Vec<i64>, i64) {
    let mut lc = vec![0u8; 256];
    if plan.materials.is_empty() {
        return (lc, Vec::new(), 0);
    }
    let mut cells: BTreeSet<u32> = BTreeSet::new();
    for indices in plan.samples.values() {
        let cell = u32::from(indices.color_idx) - 1;
        lc[cell as usize] |= 1 << indices.material_idx;
        cells.insert(cell);
    }
    let indices: Vec<i64> = cells.iter().map(|&cell| i64::from(cell)).collect();
    let current = indices.first().copied().unwrap_or(0);
    (lc, indices, current)
}
