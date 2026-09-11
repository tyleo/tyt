use crate::{
    Result, VoxjWriteOptions,
    ext::{VoxjVoxMain, to_voxj_file_with_ext},
};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{Deflate, EncodeVoxjJson, to_voxjz_file_bytes};

/// Writes a [`VoxjVoxMain`] and its `ext` block to a `.voxjz` zip archive
/// holding one compact `.voxj` member, the bytes form of
/// [`to_voxj_file_with_ext`].
pub fn to_voxjz_bytes_with_ext<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate>(
    dependencies: &D,
    state: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file_with_ext(dependencies, state, options)?;
    Ok(to_voxjz_file_bytes(dependencies, &file))
}
