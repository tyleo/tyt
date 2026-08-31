use crate::{Result, VoxjExtSlot};
use voxcore::VoxMain;
use voxj::VoxjFile;
use voxj_voxcore::from_voxj_file as raw_from_voxj_file;

/// Loads a [`VoxjFile`] into a [`VoxMain`], typing the document's `ext`
/// block into the slot `T`. The scene loads through voxj-voxcore's raw
/// conversion, then the block fills the slot through its [`VoxjExtSlot`].
pub fn from_voxj_file<T: VoxjExtSlot>(file: &VoxjFile) -> Result<VoxMain<T>> {
    let state = raw_from_voxj_file(file)?;
    let ext = T::from_voxj_ext(state.ext().as_ref())?;
    Ok(state.map_ext(|_| ext))
}
