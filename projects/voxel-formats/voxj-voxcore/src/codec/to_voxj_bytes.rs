use crate::{Result, VoxjVoxMain, VoxjWriteOptions, to_voxj_file};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{EncodeVoxjJson, to_voxj_file_bytes};

/// Writes a [`VoxjVoxMain`] to compact `.voxj` JSON bytes, the bytes form of
/// [`to_voxj_file`].
pub fn to_voxj_bytes<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson>(
    dependencies: &D,
    main: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, main, options)?;
    Ok(to_voxj_file_bytes(dependencies, &file))
}
