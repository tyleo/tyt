use crate::{GoxelVoxMain, Result, to_goxl_file};
use goxl_codec::to_gox_file_bytes;

/// Writes a [`GoxelVoxMain`] to the bytes of a Goxel `.gox` file, the inverse
/// of [`from_goxl_bytes`](crate::from_goxl_bytes). The state is written back to
/// a [`GoxlFile`](goxl::GoxlFile) and encoded with [`goxl_codec`].
pub fn to_goxl_bytes(state: &GoxelVoxMain) -> Result<Vec<u8>> {
    let file = to_goxl_file(state)?;
    Ok(to_gox_file_bytes(&file))
}
