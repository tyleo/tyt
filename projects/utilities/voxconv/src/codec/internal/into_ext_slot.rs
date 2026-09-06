use crate::Result;
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Moves a state's ext into another slot type through its block form. The
/// target slot decodes the block when it owns the block's key, and takes its
/// default when the block belongs to another format or is absent.
pub fn into_ext_slot<S: VoxExtSlot, T: VoxExtSlot>(state: VoxMain<S>) -> Result<VoxMain<T>> {
    let block = state.ext().to_vox_ext()?;

    let ext = T::from_vox_ext(block.as_ref())?;

    Ok(state.map_ext(|_| ext))
}
