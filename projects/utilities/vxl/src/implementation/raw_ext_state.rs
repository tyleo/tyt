use crate::Result;
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj_voxcore::VoxjVoxMain;

/// Re-types a format-typed state onto the verbatim block form, encoding its
/// slot through the format's [`VoxExtSlot`].
pub fn raw_ext_state<T: VoxExtSlot>(state: VoxMain<T>) -> Result<VoxjVoxMain> {
    let ext = state.ext().to_vox_ext()?;
    Ok(state.map_ext(|_| ext))
}
