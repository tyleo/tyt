use crate::{Result, VoxDocumentFile};
use goxl_voxcore::codec::{dependencies::EncodePng, to_goxl_bytes};
use voxcore::VoxMain;

/// Encodes a bare state as a `.gox` file.
pub fn write_goxl<D: EncodePng>(
    dependencies: &D,
    state: &VoxMain<()>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_goxl_bytes(dependencies, state)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
