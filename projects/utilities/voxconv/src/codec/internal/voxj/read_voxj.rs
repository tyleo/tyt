use crate::{Result, VoxDocumentFile, codec::single_file_bytes};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};
use voxj::dependencies::DecodeBase64;
use voxj_voxcore::codec::{
    dependencies::{DecodeVoxjJson, Inflate},
    from_voxj_bytes,
};

/// Decodes a `.voxj` or `.voxjz` document into a state. The `ext` block
/// lands in the state's ext directly, with no typed ext in between.
pub fn read_voxj<D: DecodeBase64 + DecodeVoxjJson + Inflate, T: VoxExtBlockCodec>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    Ok(from_voxj_bytes(dependencies, single_file_bytes(files)?)?)
}
