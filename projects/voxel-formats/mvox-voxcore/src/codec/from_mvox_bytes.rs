use crate::{Result, from_mvox_file};
use mvox_codec::from_mvox_file_bytes;
use voxcore::VoxMain;

/// Loads the bytes of a MagicaVoxel `.vox` file into a bare [`VoxMain`], the
/// bytes form of [`from_mvox_file`].
pub fn from_mvox_bytes(bytes: &[u8]) -> Result<VoxMain<()>> {
    let file = from_mvox_file_bytes(bytes)?;

    from_mvox_file(&file)
}
