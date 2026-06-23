use crate::{from_voxj_file_bytes, unwrap_voxjz};
use std::io;
use voxj::VoxjSerdeFile;

/// Decodes `.voxjz` (zip) bytes by inflating the single `.voxj` member and
/// decoding it.
pub fn from_voxjz_file_bytes(bytes: &[u8]) -> io::Result<VoxjSerdeFile> {
    from_voxj_file_bytes(&unwrap_voxjz(bytes)?)
}
