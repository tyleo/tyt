use crate::Result;
use voxj::{
    DecodeBase64,
    validation::{VoxjCheck, check_voxj_file},
};
use voxj_codec::{DecodeVoxjJson, Inflate, from_voxj_or_voxjz_file_bytes};

/// Runs every Voxel Json spec check over a `.voxj` or `.voxjz` document. The
/// checks inspect the on-disk encoding, so the document is decoded as raw
/// Voxel Json, with the container form detected from the leading bytes.
/// Undecodable bytes are an error. A document that breaks a rule reports the
/// failed check.
pub fn check_voxj_bytes<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<Vec<VoxjCheck>> {
    let file = from_voxj_or_voxjz_file_bytes(dependencies, bytes)?;

    Ok(check_voxj_file(dependencies, &file))
}
