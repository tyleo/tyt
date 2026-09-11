use crate::{Result, VoxjWriteOptions, ext::VoxjVoxMain, write_voxj};
use voxj::{CostVoxjObject, EncodeBase64, VoxjFile};

/// Encodes a [`VoxjVoxMain`] into a [`VoxjFile`] carrying its `ext` block,
/// the inverse of
/// [`from_voxj_file_with_ext`](crate::ext::from_voxj_file_with_ext) and the
/// typed form of [`to_voxj_file`](crate::to_voxj_file). An empty ext writes
/// no block, and `options` can drop the block.
pub fn to_voxj_file_with_ext<D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    state: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<VoxjFile> {
    write_voxj(dependencies, state, options)
}
