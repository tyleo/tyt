use crate::{Result, from_qb_file};
use qbcl_codec::qb::from_qb_file_bytes;
use voxcore::VoxState;

/// Loads the bytes of a Qubicle Binary `.qb` file into a [`VoxState`]. The bytes
/// are decoded with [`qbcl_codec`] and the result is loaded into voxcore's
/// in-memory form.
pub fn from_qb_bytes(bytes: &[u8]) -> Result<VoxState> {
    let file = from_qb_file_bytes(bytes)?;
    from_qb_file(&file)
}
