use crate::{
    Result, VoxjWriteOptions,
    ext::{VoxjVoxMain, to_voxj_file_with_ext},
};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{EncodeVoxjJson, to_voxj_file_bytes};

/// Writes a [`VoxjVoxMain`] and its `ext` block to compact `.voxj` JSON bytes,
/// the bytes form of [`to_voxj_file_with_ext`].
pub fn to_voxj_bytes_with_ext<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson>(
    dependencies: &D,
    state: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file_with_ext(dependencies, state, options)?;
    Ok(to_voxj_file_bytes(dependencies, &file))
}
