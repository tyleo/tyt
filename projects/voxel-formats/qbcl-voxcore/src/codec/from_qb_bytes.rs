use crate::{QubicleQbVoxMain, Result, from_qb_file};
use qbcl_codec::qb::from_qb_file_bytes;

/// Loads the bytes of a Qubicle Binary `.qb` file into a [`QubicleQbVoxMain`],
/// the bytes form of [`from_qb_file`].
pub fn from_qb_bytes(bytes: &[u8]) -> Result<QubicleQbVoxMain> {
    let file = from_qb_file_bytes(bytes)?;
    from_qb_file(&file)
}
