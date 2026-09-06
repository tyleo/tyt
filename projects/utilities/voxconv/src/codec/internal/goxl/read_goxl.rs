use crate::{
    Result, VoxDocumentFile,
    codec::{into_ext_slot, single_file_bytes},
};
use goxl_voxcore::codec::{dependencies::DecodePng, from_goxl_bytes};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Decodes a `.gox` file into a state.
pub fn read_goxl<D: DecodePng, T: VoxExtSlot>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    into_ext_slot(from_goxl_bytes(dependencies, single_file_bytes(files)?)?)
}
