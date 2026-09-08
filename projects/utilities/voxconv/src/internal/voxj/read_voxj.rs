use crate::{Result, VoxDocumentFile, single_file_bytes};
use voxcore::VoxMain;
use voxj::dependencies::DecodeBase64;
use voxj_voxcore::codec::{
    dependencies::{DecodeVoxjJson, Inflate},
    from_voxj_bytes,
};

/// Decodes a `.voxj` or `.voxjz` document into a bare state. The `ext` block
/// drops.
pub fn read_voxj<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<()>> {
    Ok(from_voxj_bytes(dependencies, single_file_bytes(files)?)?)
}
