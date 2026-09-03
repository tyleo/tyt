use crate::{Result, VOXJ_DEPENDENCIES, VoxjCheck};
use voxj::validation::check_voxj_file;
use voxj_codec::from_voxj_or_voxjz_file_bytes;

/// Runs every Voxel Json spec check over a `.voxj` or `.voxjz` document and
/// reports how it fared on each. The checks inspect the on-disk encoding, so
/// the document is decoded as raw Voxel Json. The container form is detected
/// from the leading bytes. Undecodable bytes are an error; a document that
/// breaks a rule reports the failed check.
pub fn check_voxj_bytes(bytes: &[u8]) -> Result<Vec<VoxjCheck>> {
    let file = from_voxj_or_voxjz_file_bytes(&VOXJ_DEPENDENCIES, bytes)?;

    Ok(check_voxj_file(&VOXJ_DEPENDENCIES, &file))
}
