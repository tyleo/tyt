use crate::{
    Result, VoxDocumentFile,
    ext::{VoxconvVoxMain, voxj_vox_ext_from_ext},
    voxj::{VoxjSerialization, VoxjWriteOptions},
};
use voxj::dependencies::{CostVoxjObject, EncodeBase64};
use voxj_voxcore::codec::{
    dependencies::{Deflate, EncodeVoxjJson},
    to_voxj_bytes_with_ext, to_voxj_pretty_bytes_with_ext, to_voxjz_bytes_with_ext,
};

/// Encodes a boxed state as a Voxel Json document, its ext encoded as the
/// `ext` block.
pub fn write_voxj_with_ext<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate>(
    dependencies: &D,
    state: VoxconvVoxMain,
    serialization: VoxjSerialization,
    options: &VoxjWriteOptions,
) -> Result<Vec<VoxDocumentFile>> {
    let ext = voxj_vox_ext_from_ext(state.ext().as_ref())?;
    let state = state.take_ext().0.put_ext(ext);
    let bytes = match serialization {
        VoxjSerialization::Compact => to_voxj_bytes_with_ext(dependencies, &state, options)?,
        VoxjSerialization::Pretty => to_voxj_pretty_bytes_with_ext(dependencies, &state, options)?,
        VoxjSerialization::Zip => to_voxjz_bytes_with_ext(dependencies, &state, options)?,
    };
    Ok(vec![VoxDocumentFile::single(bytes)])
}
