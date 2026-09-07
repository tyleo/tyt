use crate::{Result, VoxDocumentFile, codec::retype_ext};
use goxl_voxcore::codec::{dependencies::EncodePng, to_goxl_bytes};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a `.gox` file.
pub fn write_goxl<D: EncodePng, T: VoxExtBlockCodec>(
    dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_goxl_bytes(dependencies, &retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
