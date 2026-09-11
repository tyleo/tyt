use crate::{Result, ext::QbclVoxMain, read_qbcl};
use qbcl::qbcl::QbclFile;

/// Loads a decoded Qubicle Construction Library [`QbclFile`] into a
/// [`QbclVoxMain`] carrying its [`QbclExt`](crate::ext::QbclExt), the typed
/// form of [`from_qbcl_file`](crate::from_qbcl_file). The state writes back
/// exactly through
/// [`to_qbcl_file_with_ext`](crate::ext::to_qbcl_file_with_ext).
pub fn from_qbcl_file_with_ext(file: &QbclFile) -> Result<QbclVoxMain> {
    read_qbcl(file)
}
