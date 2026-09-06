use crate::{
    Result, VoxDocumentFile,
    codec::{into_ext_slot, single_file_bytes},
};
use qbcl_voxcore::codec::from_qb_bytes;
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Decodes a `.qb` file into a state.
pub fn read_qb<T: VoxExtSlot>(files: &[VoxDocumentFile]) -> Result<VoxMain<T>> {
    into_ext_slot(from_qb_bytes(single_file_bytes(files)?)?)
}
