use crate::{to_voxj_bytes, wrap_voxjz};
use voxj::VoxjFile;

/// Serializes `file` to a `.voxjz` zip archive holding one compact `.voxj`
/// member.
pub fn to_voxjz_bytes(file: &VoxjFile) -> serde_json::Result<Vec<u8>> {
    Ok(wrap_voxjz(&to_voxj_bytes(file)?))
}
