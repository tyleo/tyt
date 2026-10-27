use std::io::Result as IOResult;

/// Shows images inline in the terminal.
pub trait DisplayImage {
    /// Shows the 8-bit straight-alpha RGBA image `rgba`, `width` by `height`
    /// pixels, inline at the cursor and scaled to fit the terminal.
    fn display_image(&self, width: u32, height: u32, rgba: &[u8]) -> IOResult<()>;
}
