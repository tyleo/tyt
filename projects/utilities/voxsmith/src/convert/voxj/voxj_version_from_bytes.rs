use crate::Result;
use voxj_codec::from_voxj_or_voxjz_file_bytes;

/// The format version a `.voxj` or `.voxjz` document is stamped with, read
/// off the document since a loaded state drops it. The container form is
/// detected from the leading bytes.
pub fn voxj_version_from_bytes(bytes: &[u8]) -> Result<u32> {
    Ok(from_voxj_or_voxjz_file_bytes(bytes)?.version)
}
