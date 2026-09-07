use crate::{Result, check_from_voxj};
use voxcore::check::VoxCheck;
use voxj::dependencies::DecodeBase64;
use voxj_voxcore::codec::{
    check_voxj_bytes,
    dependencies::{DecodeVoxjJson, Inflate},
};

/// Runs every Voxel Json spec check over a `.voxj` or `.voxjz` document and
/// reports each in voxcore's form.
pub fn check_voxj<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<Vec<VoxCheck>> {
    Ok(check_voxj_bytes(dependencies, bytes)?
        .into_iter()
        .map(check_from_voxj)
        .collect())
}
