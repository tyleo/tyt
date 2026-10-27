use crate::test_utilities::HookRecorder;
use ty_math::TyVector3U32;
use voxcore::VoxMain;

/// The first object's live voxels as `(position, material index)` in its
/// first layer, in raster order.
pub fn live_cells(main: &VoxMain<HookRecorder>) -> Vec<(TyVector3U32, u32)> {
    let (_, object) = main.iter_objects().next().unwrap();

    let (layer_id, _) = object.iter_layers().next().unwrap();

    object
        .iter_live()
        .map(|voxel_id| {
            (
                object.voxel_position(voxel_id).unwrap(),
                object.voxel_material(voxel_id, layer_id).unwrap().to_u32(),
            )
        })
        .collect()
}
