use crate::{Result, VoxDocumentFile, retype_ext, single_file_bytes};
use goxl_voxcore::codec::{dependencies::DecodePng, from_goxl_bytes_with_ext};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Decodes a `.gox` file into a state.
pub fn read_goxl<D: DecodePng, T: VoxExtBlockCodec>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    retype_ext(from_goxl_bytes_with_ext(
        dependencies,
        single_file_bytes(files)?,
    )?)
}
