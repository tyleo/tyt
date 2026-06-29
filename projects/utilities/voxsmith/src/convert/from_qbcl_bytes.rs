use crate::{Result, from_qbcl_file};
use qbcl_codec::qbcl::from_qbcl_file_bytes;
use voxcore::VoxMain;

/// Loads the bytes of a Qubicle Construction Library `.qbcl` file into a
/// [`VoxMain`]. The bytes are decoded with [`qbcl_codec`] and the result is
/// loaded into voxcore's in-memory form.
pub fn from_qbcl_bytes(bytes: &[u8]) -> Result<VoxMain> {
    let file = from_qbcl_file_bytes(bytes)?;
    from_qbcl_file(&file)
}
