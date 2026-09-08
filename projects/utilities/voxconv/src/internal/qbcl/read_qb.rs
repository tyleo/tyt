use crate::{Result, VoxDocumentFile, retype_ext, single_file_bytes};
use qbcl_voxcore::codec::from_qb_bytes_with_ext;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Decodes a `.qb` file into a state.
pub fn read_qb<T: VoxExtBlockCodec>(files: &[VoxDocumentFile]) -> Result<VoxMain<T>> {
    retype_ext(from_qb_bytes_with_ext(single_file_bytes(files)?)?)
}
