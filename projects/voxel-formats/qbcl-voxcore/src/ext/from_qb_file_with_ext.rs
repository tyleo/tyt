use crate::{Result, ext::QbVoxMain, read_qb};
use qbcl::qb::QbFile;

/// Loads a decoded Qubicle Binary [`QbFile`] into a [`QbVoxMain`] carrying
/// its [`QbExt`](crate::ext::QbExt), the typed form of
/// [`from_qb_file`](crate::from_qb_file). The state writes back exactly
/// through [`to_qb_file_with_ext`](crate::ext::to_qb_file_with_ext).
pub fn from_qb_file_with_ext(file: &QbFile) -> Result<QbVoxMain> {
    let (state, qb_ext) = read_qb(file)?;

    Ok(state.map_ext(|()| Some(qb_ext)))
}
