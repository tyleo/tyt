use crate::Result;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Moves a state's ext into another ext type through its block form. The
/// target decodes the block when it owns the block's key, and takes its
/// default when the block belongs to another format or is absent.
pub fn retype_ext<S: VoxExtBlockCodec, T: VoxExtBlockCodec>(
    state: VoxMain<S>,
) -> Result<VoxMain<T>> {
    let block = state.ext().to_vox_ext_block()?;

    let ext = T::from_vox_ext_block(block.as_ref())?;

    Ok(state.map_ext(|_| ext))
}
