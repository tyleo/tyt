use crate::{Result, VoxDocumentFile, retype_ext, single_file_bytes};
use mvox_voxcore::codec::from_mvox_bytes;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Decodes a `.vox` file into a state. The codec needs no dependencies.
pub fn read_mvox<D, T: VoxExtBlockCodec>(
    _dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    retype_ext(from_mvox_bytes(single_file_bytes(files)?)?)
}
