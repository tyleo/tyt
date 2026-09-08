use crate::{Result, to_qb_file};
use qbcl_codec::qb::to_qb_file_bytes;
use voxcore::VoxMain;

/// Writes a bare [`VoxMain`] to the bytes of a Qubicle Binary `.qb` file, the
/// bytes form of [`to_qb_file`] and the inverse of
/// [`from_qb_bytes`](crate::codec::from_qb_bytes). Errors as [`to_qb_file`]
/// does.
pub fn to_qb_bytes(state: &VoxMain<()>) -> Result<Vec<u8>> {
    let file = to_qb_file(state)?;

    Ok(to_qb_file_bytes(&file))
}
