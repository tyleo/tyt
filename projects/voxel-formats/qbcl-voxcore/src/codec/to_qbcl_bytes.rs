use crate::{Result, to_qbcl_file};
use qbcl_codec::{CompressZlib, qbcl::to_qbcl_file_bytes};
use voxcore::VoxMain;

/// Writes a bare [`VoxMain`] to the bytes of a Qubicle Construction Library
/// `.qbcl` file through `dependencies`, the bytes form of [`to_qbcl_file`]
/// and the inverse of [`from_qbcl_bytes`](crate::codec::from_qbcl_bytes).
pub fn to_qbcl_bytes<D: CompressZlib>(dependencies: &D, state: &VoxMain<()>) -> Result<Vec<u8>> {
    let file = to_qbcl_file(state)?;

    Ok(to_qbcl_file_bytes(dependencies, &file))
}
