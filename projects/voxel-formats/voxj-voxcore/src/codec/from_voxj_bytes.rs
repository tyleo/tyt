use crate::{Result, from_voxj_file};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::DecodeBase64;
use voxj_codec::{DecodeVoxjJson, Inflate, from_voxj_or_voxjz_file_bytes};

/// Loads a `.voxj` or `.voxjz` document into a [`VoxMain`], typing the
/// document's `ext` block into the slot `T`. The container form is detected
/// from the leading bytes.
pub fn from_voxj_bytes<T: VoxExtSlot, D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VoxMain<T>> {
    let file = from_voxj_or_voxjz_file_bytes(dependencies, bytes)?;
    from_voxj_file(dependencies, &file)
}
