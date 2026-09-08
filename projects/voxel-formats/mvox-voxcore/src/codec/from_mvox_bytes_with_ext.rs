use crate::{
    Result,
    ext::{MVoxVoxMain, from_mvox_file_with_ext},
};
use mvox_codec::from_mvox_file_bytes;

/// Loads the bytes of a MagicaVoxel `.vox` file into a [`MVoxVoxMain`]
/// carrying its ext, the bytes form of [`from_mvox_file_with_ext`].
pub fn from_mvox_bytes_with_ext(bytes: &[u8]) -> Result<MVoxVoxMain> {
    let file = from_mvox_file_bytes(bytes)?;

    from_mvox_file_with_ext(&file)
}
