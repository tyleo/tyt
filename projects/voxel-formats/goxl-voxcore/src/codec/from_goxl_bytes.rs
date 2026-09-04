use crate::{GoxlVoxMain, Result, from_goxl_file};
use goxl_codec::{DecodePng, from_gox_file_bytes};

/// Loads the bytes of a Goxel `.gox` file into a [`GoxlVoxMain`] through
/// `dependencies`, the bytes form of [`from_goxl_file`].
pub fn from_goxl_bytes<D: DecodePng>(dependencies: &D, bytes: &[u8]) -> Result<GoxlVoxMain> {
    let file = from_gox_file_bytes(dependencies, bytes)?;
    from_goxl_file(&file)
}
