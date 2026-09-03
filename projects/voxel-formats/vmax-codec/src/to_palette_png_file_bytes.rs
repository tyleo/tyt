use crate::{EncodePng, Error, Result};
use vmax::VMaxPalettePngFile;

/// Encodes a [`VMaxPalettePngFile`] into `palette*.png` bytes through
/// `dependencies`: a `len x 1` RGBA PNG, one pixel per color cell, in the
/// shape Voxel Max's encoder writes. The inverse of
/// [`from_palette_png_file_bytes`](crate::from_palette_png_file_bytes).
pub fn to_palette_png_file_bytes<D: EncodePng>(
    dependencies: &D,
    file: &VMaxPalettePngFile,
) -> Result<Vec<u8>> {
    dependencies.encode_palette_png(file).map_err(Error::Png)
}
