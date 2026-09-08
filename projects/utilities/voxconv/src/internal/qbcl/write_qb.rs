use crate::{Result, VoxDocumentFile, retype_ext};
use qbcl_voxcore::codec::to_qb_bytes_with_ext;
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state as a `.qb` file.
pub fn write_qb<T: VoxExt>(state: VoxMain<T>) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qb_bytes_with_ext(&retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
