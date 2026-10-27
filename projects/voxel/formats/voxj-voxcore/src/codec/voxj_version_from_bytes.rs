use crate::Result;
use voxj_codec::{DecodeVoxjJson, Inflate, from_voxj_or_voxjz_file_bytes};

/// The format version of a `.voxj` or `.voxjz` document, read from the bytes
/// because a loaded state drops it. The container form is detected from the
/// leading bytes.
pub fn voxj_version_from_bytes<D: DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<u32> {
    Ok(from_voxj_or_voxjz_file_bytes(dependencies, bytes)?.version)
}
