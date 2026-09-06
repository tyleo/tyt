use crate::{
    Result, VoxDocumentFile,
    codec::{into_ext_slot, single_file_bytes},
};
use mvox_voxcore::codec::from_mvox_bytes;
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Decodes a `.vox` file into a state. The codec needs no dependencies.
pub fn read_mvox<D, T: VoxExtSlot>(
    _dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    into_ext_slot(from_mvox_bytes(single_file_bytes(files)?)?)
}
