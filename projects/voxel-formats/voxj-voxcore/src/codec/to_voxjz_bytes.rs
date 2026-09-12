use crate::{Result, VoxjVoxMain, VoxjWriteOptions, to_voxj_file};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{Deflate, EncodeVoxjJson, to_voxjz_file_bytes};

/// Writes a [`VoxjVoxMain`] to a `.voxjz` zip archive holding one compact
/// `.voxj` member, the bytes form of [`to_voxj_file`].
pub fn to_voxjz_bytes<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate>(
    dependencies: &D,
    main: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, main, options)?;
    Ok(to_voxjz_file_bytes(dependencies, &file))
}
