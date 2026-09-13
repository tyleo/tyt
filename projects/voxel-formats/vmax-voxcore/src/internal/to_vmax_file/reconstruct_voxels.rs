use crate::{PalettePlan, Result, VoxelIndices, object_layer};
use vmax::snapshots::VMaxVoxel;
use voxcore::VoxObject;

/// Re-bases the tight object's voxels to absolute model space, each with the
/// indices its material takes in `plan`. A voxel of an object with no layer
/// takes cell 1, since 0 is the empty cell, and slot 0. Errors when the object
/// has a second layer.
pub(crate) fn reconstruct_voxels(
    object: &VoxObject,
    plan: &PalettePlan,
    box_min: [i32; 3],
) -> Result<Vec<VMaxVoxel>> {
    let layer = object_layer(object)?;
    Ok(object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let indices = match layer {
                Some((layer_id, _)) => {
                    let material_id = object
                        .voxel_material(voxel_id, layer_id)
                        .expect("a live voxel samples its layer");
                    plan.indices[&material_id]
                }
                None => VoxelIndices {
                    color_idx: 1,
                    material_idx: 0,
                },
            };
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
        .collect())
}
