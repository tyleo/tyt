use crate::Result;
use vmax::VMaxQuickLookPngFile;

/// Reads `QuickLook/*.png` thumbnail bytes into a [`VMaxQuickLookPngFile`]. The
/// preview is kept verbatim (never re-encoded) so it round-trips byte for byte.
pub fn from_quick_look_png_file_bytes(bytes: &[u8]) -> Result<VMaxQuickLookPngFile> {
    Ok(VMaxQuickLookPngFile(bytes.to_vec()))
}
