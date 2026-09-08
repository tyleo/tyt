use crate::{Result, VoxDocumentFile, retype_ext};
use mvox_voxcore::codec::to_mvox_bytes_with_ext;
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state with a boxed ext as a `.vox` file. The codec needs no
/// dependencies.
pub fn write_mvox_with_ext<D>(
    _dependencies: &D,
    state: VoxMain<Box<dyn VoxExt>>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_mvox_bytes_with_ext(&retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
