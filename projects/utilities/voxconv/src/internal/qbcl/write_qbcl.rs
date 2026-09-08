use crate::{Result, VoxDocumentFile};
use qbcl_voxcore::codec::{dependencies::CompressZlib, to_qbcl_bytes};
use voxcore::VoxMain;

/// Encodes a bare state as a `.qbcl` file.
pub fn write_qbcl<D: CompressZlib>(
    dependencies: &D,
    state: &VoxMain<()>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qbcl_bytes(dependencies, state)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
