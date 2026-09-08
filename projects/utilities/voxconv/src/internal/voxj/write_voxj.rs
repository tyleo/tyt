use crate::{
    Result, VoxDocumentFile,
    voxj::{VoxjSerialization, VoxjWriteOptions},
};
use voxcore::{VoxMain, ext::VoxExt};
use voxj::dependencies::{CostVoxjObject, EncodeBase64};
use voxj_voxcore::codec::{
    dependencies::{Deflate, EncodeVoxjJson},
    to_voxj_bytes, to_voxj_pretty_bytes, to_voxjz_bytes,
};

/// Encodes a state as a Voxel Json document.
pub fn write_voxj<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate, T: VoxExt>(
    dependencies: &D,
    state: &VoxMain<T>,
    serialization: VoxjSerialization,
    options: &VoxjWriteOptions,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = match serialization {
        VoxjSerialization::Compact => to_voxj_bytes(dependencies, state, options)?,
        VoxjSerialization::Pretty => to_voxj_pretty_bytes(dependencies, state, options)?,
        VoxjSerialization::Zip => to_voxjz_bytes(dependencies, state, options)?,
    };

    Ok(vec![VoxDocumentFile::single(bytes)])
}
