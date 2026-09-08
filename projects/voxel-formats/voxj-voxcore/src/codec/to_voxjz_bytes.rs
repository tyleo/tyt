use crate::{Result, VoxjWriteOptions, to_voxj_file};
use voxcore::{VoxMain, ext::VoxExt};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{Deflate, EncodeVoxjJson, to_voxjz_file_bytes};

/// Writes a [`VoxMain`] to a `.voxjz` zip archive holding one compact
/// `.voxj` member. [`to_voxj_file`] encodes the document under `options`.
pub fn to_voxjz_bytes<T: VoxExt, D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate>(
    dependencies: &D,
    state: &VoxMain<T>,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, state, options)?;
    Ok(to_voxjz_file_bytes(dependencies, &file))
}
