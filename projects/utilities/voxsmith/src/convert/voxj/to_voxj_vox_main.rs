use crate::{Result, VoxjVoxMain};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj_voxcore::Error as VoxjError;

/// Encodes a state's slot into the `ext` block a [`VoxjVoxMain`] carries, so
/// states from different formats share one type on the way to a Voxel Json
/// write. The scene moves over unchanged.
pub fn to_voxj_vox_main<T: VoxExtSlot>(state: VoxMain<T>) -> Result<VoxjVoxMain> {
    let ext = state.ext().to_vox_ext().map_err(VoxjError::from)?;

    Ok(state.map_ext(|_| ext))
}
