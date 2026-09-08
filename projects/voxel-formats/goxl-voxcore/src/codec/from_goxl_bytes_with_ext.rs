use crate::{
    Result,
    ext::{GoxlVoxMain, from_goxl_file_with_ext},
};
use goxl_codec::{DecodePng, from_gox_file_bytes};

/// Loads the bytes of a Goxel `.gox` file through `dependencies` into a
/// [`GoxlVoxMain`] carrying its ext, the bytes form of
/// [`from_goxl_file_with_ext`].
pub fn from_goxl_bytes_with_ext<D: DecodePng>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<GoxlVoxMain> {
    let file = from_gox_file_bytes(dependencies, bytes)?;

    from_goxl_file_with_ext(&file)
}
