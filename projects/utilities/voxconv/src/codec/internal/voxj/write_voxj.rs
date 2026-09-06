use crate::{
    Result, VoxDocumentFile,
    voxj::{VoxjSerialization, VoxjWriteOptions},
};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::dependencies::{CostVoxjObject, EncodeBase64};
use voxj_voxcore::{
    VoxjFileBuilder,
    codec::{
        VoxjFileBuilderCodec,
        dependencies::{Deflate, EncodeVoxjJson},
    },
};

/// Encodes a state as a Voxel Json document. The slot's block persists as
/// the document's `ext` block.
pub fn write_voxj<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate, T: VoxExtSlot>(
    dependencies: &D,
    state: &VoxMain<T>,
    options: VoxjWriteOptions,
) -> Result<Vec<VoxDocumentFile>> {
    let builder = VoxjFileBuilder::new(dependencies, state)
        .position_encoding(options.position_encoding)
        .sample_encoding(options.sample_encoding)
        .ext(options.ext)
        .edit_state(options.edit_state);

    let bytes = match options.serialization {
        VoxjSerialization::Compact => builder.to_voxj_bytes()?,
        VoxjSerialization::Pretty => builder.to_voxj_pretty_bytes()?,
        VoxjSerialization::Zip => builder.to_voxjz_bytes()?,
    };

    Ok(vec![VoxDocumentFile::single(bytes)])
}
