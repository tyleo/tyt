use crate::{Result, VoxDocumentFile, box_ext, ext::VoxconvVoxMain, single_file_bytes};
use goxl_voxcore::codec::{dependencies::DecodePng, from_goxl_bytes_with_ext};

/// Decodes a `.gox` file into a state carrying its ext.
pub fn read_goxl_with_ext<D: DecodePng>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxconvVoxMain> {
    Ok(box_ext(from_goxl_bytes_with_ext(
        dependencies,
        single_file_bytes(files)?,
    )?))
}
