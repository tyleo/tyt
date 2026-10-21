use crate::{Deflate, EncodeVoxjJson, Result, to_voxj_file_bytes, wrap_voxjz};
use voxj::VoxjFile;

/// Serializes `file` through `dependencies` to a `.voxjz` zip archive holding
/// one compact `.voxj` member.
pub fn to_voxjz_file_bytes<D: EncodeVoxjJson + Deflate>(
    dependencies: &D,
    file: &VoxjFile,
) -> Result<Vec<u8>> {
    Ok(wrap_voxjz(
        dependencies,
        &to_voxj_file_bytes(dependencies, file)?,
    ))
}
