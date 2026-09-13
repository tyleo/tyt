use crate::PalettePlan;
use vmax::snapshots::VMaxVoxel;
use voxcore::VoxObject;

/// Re-bases the tight object's voxels to absolute model space, each with the
/// indices its sample tuple takes in `plan`.
pub(crate) fn reconstruct_voxels(
    object: &VoxObject,
    plan: &PalettePlan,
    box_min: [i32; 3],
) -> Vec<VMaxVoxel> {
    let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();
    let mut sample = Vec::with_capacity(layer_ids.len());
    object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            sample.clear();
            sample.extend(layer_ids.iter().map(|&layer_id| {
                object
                    .voxel_material(voxel_id, layer_id)
                    .expect("a live voxel samples every layer")
            }));
            let indices = plan.samples[&sample];
            VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx: indices.material_idx,
                color_idx: indices.color_idx,
            }
        })
        .collect()
}
