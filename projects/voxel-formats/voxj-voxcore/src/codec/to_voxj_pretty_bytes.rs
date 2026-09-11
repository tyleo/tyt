use crate::{Result, VoxjWriteOptions, to_voxj_file};
use voxcore::VoxMain;
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{EncodeVoxjJson, to_voxj_pretty_file_bytes};

/// Writes a bare [`VoxMain`] to pretty-printed `.voxj` JSON bytes.
/// [`to_voxj_file`] encodes the document under `options`.
pub fn to_voxj_pretty_bytes<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson>(
    dependencies: &D,
    state: &VoxMain<()>,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, state, options)?;
    Ok(to_voxj_pretty_file_bytes(dependencies, &file))
}
