use crate::{Result, VoxDocumentFile, ext::VoxconvVoxMain, find_ext, write_goxl};
use goxl_voxcore::{
    codec::{dependencies::EncodePng, to_goxl_bytes_with_ext},
    ext::GoxlExt,
};

/// Encodes a boxed state as a `.gox` file: exactly through the Goxel ext the
/// box holds or encodes, else synthesized like the bare pair.
pub fn write_goxl_with_ext<D: EncodePng>(
    dependencies: &D,
    state: VoxconvVoxMain,
) -> Result<Vec<VoxDocumentFile>> {
    let Some(ext) = find_ext::<GoxlExt>(state.ext().as_ref())? else {
        return write_goxl(dependencies, &state.take_ext().0);
    };
    let bytes = to_goxl_bytes_with_ext(dependencies, &state.take_ext().0.put_ext(ext))?;
    Ok(vec![VoxDocumentFile::single(bytes)])
}
