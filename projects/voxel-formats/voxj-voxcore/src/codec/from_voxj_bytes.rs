use crate::{Result, from_voxj_file};
use voxcore::VoxMain;
use voxj::DecodeBase64;
use voxj_codec::{DecodeVoxjJson, Inflate, from_voxj_or_voxjz_file_bytes};

/// Loads a `.voxj` or `.voxjz` document into a bare [`VoxMain`]. The `ext`
/// block drops.
/// [`from_voxj_bytes_with_ext`](crate::codec::from_voxj_bytes_with_ext)
/// keeps it. The container form is detected from the leading bytes.
pub fn from_voxj_bytes<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VoxMain<()>> {
    let file = from_voxj_or_voxjz_file_bytes(dependencies, bytes)?;
    from_voxj_file(dependencies, &file)
}
