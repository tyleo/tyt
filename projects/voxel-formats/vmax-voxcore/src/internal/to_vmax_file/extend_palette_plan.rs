use crate::{PaletteAxes, PalettePlan, Result, voxel_indices};
use voxcore::VoxObject;

/// Adds every sample tuple `object`'s live voxels draw to `plan`, assigning
/// each new one its indices.
pub(crate) fn extend_palette_plan(
    plan: &mut PalettePlan,
    axes: &PaletteAxes,
    object: &VoxObject,
) -> Result<()> {
    let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();
    let mut sample = Vec::with_capacity(layer_ids.len());
    for voxel_id in object.iter_live() {
        sample.clear();
        sample.extend(layer_ids.iter().map(|&layer_id| {
            object
                .voxel_material(voxel_id, layer_id)
                .expect("a live voxel samples every layer")
        }));
        if plan.samples.contains_key(&sample) {
            continue;
        }
        let indices = voxel_indices(plan, axes, &sample)?;
        plan.samples.insert(sample.clone(), indices);
    }
    Ok(())
}
