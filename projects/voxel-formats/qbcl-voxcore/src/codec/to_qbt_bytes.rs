use crate::{Result, to_qbt_file};
use qbcl_codec::{CompressZlib, qbt::to_qbt_file_bytes};
use voxcore::VoxMain;

/// Writes a bare [`VoxMain`] to the bytes of a Qubicle Binary Tree `.qbt`
/// file through `dependencies`, the bytes form of [`to_qbt_file`] and the
/// inverse of [`from_qbt_bytes`](crate::codec::from_qbt_bytes). Errors as
/// [`to_qbt_file`] does.
pub fn to_qbt_bytes<D: CompressZlib>(dependencies: &D, state: &VoxMain<()>) -> Result<Vec<u8>> {
    let file = to_qbt_file(state)?;

    Ok(to_qbt_file_bytes(dependencies, &file))
}
