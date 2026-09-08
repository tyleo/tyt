use crate::{Result, VoxDocumentFile, retype_ext};
use qbcl_voxcore::codec::{dependencies::CompressZlib, to_qbcl_bytes_with_ext};
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state with a boxed ext as a `.qbcl` file.
pub fn write_qbcl_with_ext<D: CompressZlib>(
    dependencies: &D,
    state: VoxMain<Box<dyn VoxExt>>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qbcl_bytes_with_ext(dependencies, &retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
