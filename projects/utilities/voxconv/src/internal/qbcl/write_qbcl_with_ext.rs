use crate::{Result, VoxDocumentFile, ext::VoxconvVoxMain, find_ext, write_qbcl};
use qbcl_voxcore::{
    codec::{dependencies::CompressZlib, to_qbcl_bytes_with_ext},
    ext::QbclExt,
};

/// Encodes a boxed state as a `.qbcl` file: exactly through the Qubicle
/// Construction Library ext the box holds or encodes, else synthesized like
/// the bare pair.
pub fn write_qbcl_with_ext<D: CompressZlib>(
    dependencies: &D,
    state: VoxconvVoxMain,
) -> Result<Vec<VoxDocumentFile>> {
    let Some(ext) = find_ext::<QbclExt>(state.ext().as_ref())? else {
        return write_qbcl(dependencies, &state.take_ext().0);
    };
    let bytes = to_qbcl_bytes_with_ext(dependencies, &state.take_ext().0.put_ext(ext))?;
    Ok(vec![VoxDocumentFile::single(bytes)])
}
