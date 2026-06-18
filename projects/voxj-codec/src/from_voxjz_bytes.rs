use crate::{from_voxj_bytes, unwrap_voxjz};
use serde_json::Value;
use std::io;
use voxj::VoxjFile;

/// Decodes `.voxjz` (zip) bytes by inflating the single `.voxj` member and
/// decoding it. Returns the document and its opaque `main.ext` namespace.
pub fn from_voxjz_bytes(bytes: &[u8]) -> io::Result<(VoxjFile, Value)> {
    from_voxj_bytes(&unwrap_voxjz(bytes)?)
}
