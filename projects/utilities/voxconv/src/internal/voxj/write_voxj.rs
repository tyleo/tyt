use crate::{
    Result, VoxDocumentFile,
    voxj::{VoxjSerialization, VoxjWriteOptions},
};
use voxcore::{VoxMain, ext::VoxExt};
use voxj::dependencies::{CostVoxjObject, EncodeBase64};
use voxj_voxcore::{
    VoxjWriteOptions as BridgeVoxjWriteOptions,
    codec::{
        dependencies::{Deflate, EncodeVoxjJson},
        to_voxj_bytes, to_voxj_pretty_bytes, to_voxjz_bytes,
    },
};

/// Encodes a state as a Voxel Json document.
pub fn write_voxj<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate, T: VoxExt>(
    dependencies: &D,
    state: &VoxMain<T>,
    options: VoxjWriteOptions,
) -> Result<Vec<VoxDocumentFile>> {
    let bridge_options = BridgeVoxjWriteOptions {
        position_encoding: options.position_encoding,
        sample_encoding: options.sample_encoding,
        ext: options.ext,
        edit_state: options.edit_state,
    };

    let bytes = match options.serialization {
        VoxjSerialization::Compact => to_voxj_bytes(dependencies, state, &bridge_options)?,
        VoxjSerialization::Pretty => to_voxj_pretty_bytes(dependencies, state, &bridge_options)?,
        VoxjSerialization::Zip => to_voxjz_bytes(dependencies, state, &bridge_options)?,
    };

    Ok(vec![VoxDocumentFile::single(bytes)])
}
