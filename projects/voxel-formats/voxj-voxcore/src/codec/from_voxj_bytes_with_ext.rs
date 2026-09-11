use crate::{
    Result,
    ext::{VoxjVoxMain, from_voxj_file_with_ext},
};
use voxj::DecodeBase64;
use voxj_codec::{DecodeVoxjJson, Inflate, from_voxj_or_voxjz_file_bytes};

/// Loads a `.voxj` or `.voxjz` document into a [`VoxjVoxMain`] carrying its
/// `ext` block as it was parsed, the bytes form of
/// [`from_voxj_file_with_ext`]. The container form is detected from the
/// leading bytes.
pub fn from_voxj_bytes_with_ext<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VoxjVoxMain> {
    let file = from_voxj_or_voxjz_file_bytes(dependencies, bytes)?;
    from_voxj_file_with_ext(dependencies, &file)
}
