use crate::{Result, from_goxl_file};
use goxl_codec::from_gox_file_bytes;
use voxcore::VoxState;

/// Loads the bytes of a Goxel `.gox` file into a [`VoxState`]. The bytes are
/// decoded with [`goxl_codec`] and the result is loaded into voxcore's in-memory
/// form.
pub fn from_goxl_bytes(bytes: &[u8]) -> Result<VoxState> {
    let file = from_gox_file_bytes(bytes)?;
    from_goxl_file(&file)
}
