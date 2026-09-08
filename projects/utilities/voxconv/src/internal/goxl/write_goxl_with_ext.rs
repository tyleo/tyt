use crate::{Result, VoxDocumentFile, retype_ext};
use goxl_voxcore::codec::{dependencies::EncodePng, to_goxl_bytes_with_ext};
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state with a boxed ext as a `.gox` file.
pub fn write_goxl_with_ext<D: EncodePng>(
    dependencies: &D,
    state: VoxMain<Box<dyn VoxExt>>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_goxl_bytes_with_ext(dependencies, &retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
