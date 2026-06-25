use crate::{Result, from_qbt_file};
use qbcl_codec::qbt::from_qbt_file_bytes;
use voxcore::VoxState;

/// Loads the bytes of a Qubicle Binary Tree `.qbt` file into a [`VoxState`]. The
/// bytes are decoded with [`qbcl_codec`] and the result is loaded into voxcore's
/// in-memory form.
pub fn from_qbt_bytes(bytes: &[u8]) -> Result<VoxState> {
    let file = from_qbt_file_bytes(bytes)?;
    from_qbt_file(&file)
}
