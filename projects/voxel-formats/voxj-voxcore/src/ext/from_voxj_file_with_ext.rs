use crate::{Result, ext::VoxjVoxMain, read_voxj};
use voxj::{DecodeBase64, VoxjFile};

/// Loads a [`VoxjFile`] into a [`VoxjVoxMain`] carrying its `ext` block as it
/// was parsed, the typed form of [`from_voxj_file`](crate::from_voxj_file). A
/// document with no block loads an empty ext. The state writes back exactly
/// through [`to_voxj_file_with_ext`](crate::ext::to_voxj_file_with_ext).
///
/// Errors if:
///
/// 1. [`from_voxj_file`](crate::from_voxj_file) errors
/// 2. the `ext` block holds a non-finite number or a repeated key
pub fn from_voxj_file_with_ext<D: DecodeBase64>(
    dependencies: &D,
    file: &VoxjFile,
) -> Result<VoxjVoxMain> {
    read_voxj(dependencies, file)
}
