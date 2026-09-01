use crate::{Result, to_voxj_file};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::to_voxj_file_bytes;

/// Writes a [`VoxMain`] to compact `.voxj` JSON bytes, choosing each object's
/// block encodings by the lowest cost. The document is stamped with the
/// current voxj format version.
pub fn to_voxj_bytes<T: VoxExtSlot, D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    state: &VoxMain<T>,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, state)?;
    Ok(to_voxj_file_bytes(&file)?)
}
