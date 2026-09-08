use crate::{Result, VoxDocumentFile, retype_ext};
use goxl_voxcore::codec::{dependencies::EncodePng, to_goxl_bytes_with_ext};
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state as a `.gox` file.
pub fn write_goxl<D: EncodePng, T: VoxExt>(
    dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_goxl_bytes_with_ext(dependencies, &retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
