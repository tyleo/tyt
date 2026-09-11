use crate::{Result, VoxDocumentFile, ext::VoxconvVoxMain, find_ext, write_qbt};
use qbcl_voxcore::{
    codec::{dependencies::CompressZlib, to_qbt_bytes_with_ext},
    ext::QbtExt,
};

/// Encodes a boxed state as a `.qbt` file: exactly through the Qubicle
/// Binary Tree ext the box holds or encodes, else synthesized like the bare
/// pair.
pub fn write_qbt_with_ext<D: CompressZlib>(
    dependencies: &D,
    state: VoxconvVoxMain,
) -> Result<Vec<VoxDocumentFile>> {
    let Some(ext) = find_ext::<QbtExt>(state.ext().as_ref())? else {
        return write_qbt(dependencies, &state.take_ext().0);
    };
    let bytes = to_qbt_bytes_with_ext(dependencies, &state.take_ext().0.put_ext(ext))?;
    Ok(vec![VoxDocumentFile::single(bytes)])
}
