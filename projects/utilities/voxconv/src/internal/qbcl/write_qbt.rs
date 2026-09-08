use crate::{Result, VoxDocumentFile};
use qbcl_voxcore::codec::{dependencies::CompressZlib, to_qbt_bytes};
use voxcore::VoxMain;

/// Encodes a bare state as a `.qbt` file.
pub fn write_qbt<D: CompressZlib>(
    dependencies: &D,
    state: &VoxMain<()>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qbt_bytes(dependencies, state)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
