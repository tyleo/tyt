use crate::{Result, VoxDocumentFile, ext::VoxconvVoxMain, find_ext, write_qb};
use qbcl_voxcore::{codec::to_qb_bytes_with_ext, ext::QbExt};

/// Encodes a boxed state as a `.qb` file: exactly through the Qubicle Binary
/// ext the box holds or encodes, else synthesized like the bare pair.
pub fn write_qb_with_ext(state: VoxconvVoxMain) -> Result<Vec<VoxDocumentFile>> {
    let Some(ext) = find_ext::<QbExt>(state.ext().as_ref())? else {
        return write_qb(&state.take_ext().0);
    };
    let bytes = to_qb_bytes_with_ext(&state.take_ext().0.put_ext(ext))?;
    Ok(vec![VoxDocumentFile::single(bytes)])
}
