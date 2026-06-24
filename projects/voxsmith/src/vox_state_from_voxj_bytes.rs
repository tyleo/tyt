use crate::vox_state_from_voxj_codec_main;
use std::io;
use voxcore::VoxState;
use voxj_codec::{decode_voxj_file, from_voxj_or_voxjz_file_bytes};

/// Loads a `.voxj` or `.voxjz` document into a [`VoxState`]. The container form
/// is detected from the leading bytes, the encoded geometry is decoded, and the
/// result is loaded into voxcore's in-memory form.
pub fn vox_state_from_voxj_bytes(bytes: &[u8]) -> io::Result<VoxState> {
    let serde_file = from_voxj_or_voxjz_file_bytes(bytes).map_err(io::Error::other)?;
    let codec_file = decode_voxj_file(&serde_file).map_err(io::Error::other)?;
    vox_state_from_voxj_codec_main(&codec_file.main)
}
