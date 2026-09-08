use crate::{
    Result,
    ext::{QbVoxMain, from_qb_file_with_ext},
};
use qbcl_codec::qb::from_qb_file_bytes;

/// Loads the bytes of a Qubicle Binary `.qb` file into a [`QbVoxMain`]
/// carrying its ext, the bytes form of [`from_qb_file_with_ext`].
pub fn from_qb_bytes_with_ext(bytes: &[u8]) -> Result<QbVoxMain> {
    let file = from_qb_file_bytes(bytes)?;

    from_qb_file_with_ext(&file)
}
