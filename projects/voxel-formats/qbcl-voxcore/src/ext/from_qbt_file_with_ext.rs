use crate::{Result, ext::QbtVoxMain, read_qbt};
use qbcl::qbt::QbtFile;

/// Loads a decoded Qubicle Binary Tree [`QbtFile`] into a [`QbtVoxMain`]
/// carrying its [`QbtExt`](crate::ext::QbtExt), the typed form of
/// [`from_qbt_file`](crate::from_qbt_file). The state writes back exactly
/// through [`to_qbt_file_with_ext`](crate::ext::to_qbt_file_with_ext).
pub fn from_qbt_file_with_ext(file: &QbtFile) -> Result<QbtVoxMain> {
    read_qbt(file)
}
