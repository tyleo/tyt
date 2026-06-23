use std::io::Result;
use vmax::VMaxQuickLookPngFile;

/// Writes a [`VMaxQuickLookPngFile`] back to `QuickLook/*.png` bytes — the inverse
/// of [`from_quick_look_png_file_bytes`](crate::from_quick_look_png_file_bytes).
/// The preserved preview is returned verbatim.
pub fn to_quick_look_png_file_bytes(file: &VMaxQuickLookPngFile) -> Result<Vec<u8>> {
    Ok(file.0.clone())
}
