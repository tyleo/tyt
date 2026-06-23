use crate::{from_voxj_file_bytes, from_voxjz_file_bytes};
use std::io;
use voxj::VoxjSerdeFile;

/// Decodes either a `.voxj` (JSON, leading `{`) or `.voxjz` (zip, leading `PK`)
/// document, detecting the container by its leading bytes.
pub fn from_voxj_or_voxjz_file_bytes(bytes: &[u8]) -> io::Result<VoxjSerdeFile> {
    if bytes.starts_with(b"PK") {
        from_voxjz_file_bytes(bytes)
    } else {
        from_voxj_file_bytes(bytes)
    }
}
