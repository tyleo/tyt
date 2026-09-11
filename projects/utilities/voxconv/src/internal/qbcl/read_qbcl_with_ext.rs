use crate::{Result, VoxDocumentFile, box_ext, ext::VoxconvVoxMain, single_file_bytes};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbcl_bytes_with_ext};

/// Decodes a `.qbcl` file into a state carrying its ext.
pub fn read_qbcl_with_ext<D: DecompressZlib>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxconvVoxMain> {
    Ok(box_ext(from_qbcl_bytes_with_ext(
        dependencies,
        single_file_bytes(files)?,
    )?))
}
