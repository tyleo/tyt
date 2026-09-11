use crate::{Result, VoxDocumentFile, box_ext, ext::VoxconvVoxMain, single_file_bytes};
use mvox_voxcore::codec::from_mvox_bytes_with_ext;

/// Decodes a `.vox` file into a state carrying its ext. The codec needs no
/// dependencies.
pub fn read_mvox_with_ext<D>(
    _dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxconvVoxMain> {
    Ok(box_ext(from_mvox_bytes_with_ext(single_file_bytes(
        files,
    )?)?))
}
