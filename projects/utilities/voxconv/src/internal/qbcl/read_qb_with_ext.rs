use crate::{Result, VoxDocumentFile, box_ext, ext::VoxconvVoxMain, single_file_bytes};
use qbcl_voxcore::codec::from_qb_bytes_with_ext;

/// Decodes a `.qb` file into a state carrying its ext.
pub fn read_qb_with_ext(files: &[VoxDocumentFile]) -> Result<VoxconvVoxMain> {
    Ok(box_ext(from_qb_bytes_with_ext(single_file_bytes(files)?)?))
}
