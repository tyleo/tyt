use crate::{from_voxj_bytes, from_voxjz_bytes};
use serde_json::Value;
use std::io;
use voxj::VoxjFile;

/// Decodes either a `.voxj` (JSON, leading `{`) or `.voxjz` (zip, leading `PK`)
/// document, detecting the container by its leading bytes.
pub fn from_voxj_or_voxjz_bytes(bytes: &[u8]) -> io::Result<(VoxjFile, Value)> {
    if bytes.starts_with(b"PK") {
        from_voxjz_bytes(bytes)
    } else {
        from_voxj_bytes(bytes)
    }
}
