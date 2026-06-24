use crate::voxj_codec_main_from_vox_state;
use std::io;
use voxcore::VoxState;
use voxj::{VoxjCodecFile, VoxjSerdeFile};
use voxj_codec::encode_voxj_file_smallest;

/// The voxj format version stamped on documents written from a [`VoxState`],
/// which does not itself carry a version.
pub(crate) const VOXJ_FORMAT_VERSION: u32 = 1;

/// Encodes a [`VoxState`] into a [`VoxjSerdeFile`] with the smallest per-object
/// block encodings, the shared step behind the `.voxj` and `.voxjz` writers.
pub(crate) fn voxj_serde_file_from_vox_state(state: &VoxState) -> io::Result<VoxjSerdeFile> {
    let file = VoxjCodecFile {
        version: VOXJ_FORMAT_VERSION,
        main: voxj_codec_main_from_vox_state(state),
    };
    encode_voxj_file_smallest(&file).map_err(io::Error::other)
}
