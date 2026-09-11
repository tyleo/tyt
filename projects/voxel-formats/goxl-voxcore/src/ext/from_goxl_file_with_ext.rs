use crate::{Result, ext::GoxlVoxMain, read_goxl};
use goxl::GoxlFile;

/// Loads a Goxel [`GoxlFile`] into a [`GoxlVoxMain`] carrying its
/// [`GoxlExt`](crate::ext::GoxlExt), the typed form of
/// [`from_goxl_file`](crate::from_goxl_file). The state writes back exactly
/// through [`to_goxl_file_with_ext`](crate::ext::to_goxl_file_with_ext).
pub fn from_goxl_file_with_ext(file: &GoxlFile) -> Result<GoxlVoxMain> {
    read_goxl(file)
}
