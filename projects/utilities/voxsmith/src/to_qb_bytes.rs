use crate::{Result, to_qb_file};
use qbcl_codec::qb::to_qb_file_bytes;
use voxcore::VoxMain;

/// Writes a [`VoxMain`] to the bytes of a Qubicle Binary `.qb` file, the inverse
/// of [`from_qb_bytes`](crate::from_qb_bytes). The state is written back to a
/// [`QbFile`](qbcl::qb::QbFile) and encoded with [`qbcl_codec`].
pub fn to_qb_bytes(state: &VoxMain) -> Result<Vec<u8>> {
    let file = to_qb_file(state)?;
    Ok(to_qb_file_bytes(&file))
}
