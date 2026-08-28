use crate::{QubicleQbVoxMain, Result, from_qb_file};
use qbcl_codec::qb::from_qb_file_bytes;

/// Loads the bytes of a Qubicle Binary `.qb` file into a [`QubicleQbVoxMain`].
/// The bytes are decoded with [`qbcl_codec`] and the result is loaded into
/// voxcore's in-memory form.
pub fn from_qb_bytes(bytes: &[u8]) -> Result<QubicleQbVoxMain> {
    let file = from_qb_file_bytes(bytes)?;
    from_qb_file(&file)
}
