use crate::{Result, VoxDocumentFile, box_ext, single_file_bytes};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbt_bytes_with_ext};
use voxcore::{VoxMain, ext::VoxExt};

/// Decodes a `.qbt` file into a state carrying its ext.
pub fn read_qbt_with_ext<D: DecompressZlib>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<Box<dyn VoxExt>>> {
    Ok(box_ext(from_qbt_bytes_with_ext(
        dependencies,
        single_file_bytes(files)?,
    )?))
}
