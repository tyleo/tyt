use crate::{Result, VoxDocumentFile, single_file_bytes};
use goxl_voxcore::codec::{dependencies::DecodePng, from_goxl_bytes};
use voxcore::VoxMain;

/// Decodes a `.gox` file into a bare state.
pub fn read_goxl<D: DecodePng>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
    Ok(from_goxl_bytes(dependencies, single_file_bytes(files)?)?)
}
