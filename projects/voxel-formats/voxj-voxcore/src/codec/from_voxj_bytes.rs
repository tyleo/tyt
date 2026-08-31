use crate::{Result, VoxjVoxMain, from_voxj_file};
use voxj_codec::from_voxj_or_voxjz_file_bytes;

/// Loads a `.voxj` or `.voxjz` document into a [`VoxjVoxMain`]. The
/// container form is detected from the leading bytes, then each object's
/// encoded geometry is decoded and the result is loaded into voxcore's
/// in-memory form.
pub fn from_voxj_bytes(bytes: &[u8]) -> Result<VoxjVoxMain> {
    let file = from_voxj_or_voxjz_file_bytes(bytes)?;
    from_voxj_file(&file)
}
