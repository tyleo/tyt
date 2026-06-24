use crate::{Result, to_voxj_file_bytes, wrap_voxjz};
use voxj::VoxjSerdeFile;

/// Serializes `file` to a `.voxjz` zip archive holding one compact `.voxj`
/// member.
pub fn to_voxjz_file_bytes(file: &VoxjSerdeFile) -> Result<Vec<u8>> {
    Ok(wrap_voxjz(&to_voxj_file_bytes(file)?))
}
