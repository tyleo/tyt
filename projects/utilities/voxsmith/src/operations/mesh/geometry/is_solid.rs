use ty_math::TyVector3U32;
use voxcore::VoxObject;

/// Whether the grid cell at `position` holds a live voxel. A cell outside
/// the build volume is empty.
pub(crate) fn is_solid(object: &VoxObject, position: [i64; 3]) -> bool {
    let bounds = object.bounds().to_array();

    if position
        .iter()
        .zip(bounds)
        .any(|(&component, bound)| component < 0 || component >= bound as i64)
    {
        return false;
    }

    let position = TyVector3U32::new(position[0] as u32, position[1] as u32, position[2] as u32);

    object
        .voxel_id(position)
        .is_some_and(|voxel_id| object.is_live(voxel_id))
}
