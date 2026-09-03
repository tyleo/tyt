use crate::{Result, to_voxj_file};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{Deflate, EncodeVoxjJson, to_voxjz_file_bytes};

/// Writes a [`VoxMain`] to a `.voxjz` zip archive holding one compact
/// `.voxj` member, choosing each object's block encodings by the lowest
/// cost. The document is stamped with the current voxj format version.
pub fn to_voxjz_bytes<
    T: VoxExtSlot,
    D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate,
>(
    dependencies: &D,
    state: &VoxMain<T>,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, state)?;
    Ok(to_voxjz_file_bytes(dependencies, &file))
}
