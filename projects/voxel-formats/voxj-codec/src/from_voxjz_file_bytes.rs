use crate::{DecodeVoxjJson, Inflate, Result, from_voxj_file_bytes, unwrap_voxjz};
use voxj::VoxjFile;

/// Decodes `.voxjz` (zip) bytes by inflating the single `.voxj` member and
/// decoding it, both through `dependencies`.
pub fn from_voxjz_file_bytes<D: DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VoxjFile> {
    from_voxj_file_bytes(dependencies, &unwrap_voxjz(dependencies, bytes)?)
}
