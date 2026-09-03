use crate::{DecodeVoxjJson, Inflate, Result, from_voxj_file_bytes, from_voxjz_file_bytes};
use voxj::VoxjFile;

/// Decodes either a `.voxj` (JSON, leading `{`) or `.voxjz` (zip, leading `PK`)
/// document through `dependencies`, detecting the container by its leading
/// bytes.
pub fn from_voxj_or_voxjz_file_bytes<D: DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VoxjFile> {
    if bytes.starts_with(b"PK") {
        from_voxjz_file_bytes(dependencies, bytes)
    } else {
        from_voxj_file_bytes(dependencies, bytes)
    }
}
