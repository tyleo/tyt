use crate::{Result, VoxDocumentFile, ext::VoxconvVoxMain, find_ext, write_mvox};
use mvox_voxcore::{codec::to_mvox_bytes_with_ext, ext::MVoxExt};

/// Encodes a boxed state as a `.vox` file: exactly through the MagicaVoxel
/// ext the box holds or encodes, else synthesized like the bare pair. The
/// codec needs no dependencies.
pub fn write_mvox_with_ext<D>(
    dependencies: &D,
    state: VoxconvVoxMain,
) -> Result<Vec<VoxDocumentFile>> {
    let Some(ext) = find_ext::<MVoxExt>(state.ext().as_ref())? else {
        return write_mvox(dependencies, &state.take_ext().0);
    };
    let bytes = to_mvox_bytes_with_ext(&state.take_ext().0.put_ext(ext))?;
    Ok(vec![VoxDocumentFile::single(bytes)])
}
