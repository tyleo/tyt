use crate::Result;
use voxcore::VoxMain;
use voxj_voxcore::VoxjVoxMain;
use voxsmith::VoxjExtSlot;

/// Re-types a format-typed state onto the verbatim block form, encoding its
/// slot through the format's [`VoxjExtSlot`].
pub fn raw_ext_state<T: VoxjExtSlot>(state: VoxMain<T>) -> Result<VoxjVoxMain> {
    let ext = state.ext().to_voxj_ext()?;
    Ok(state.map_ext(|_| ext))
}
