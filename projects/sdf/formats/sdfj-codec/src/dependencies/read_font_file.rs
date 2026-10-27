use std::{io::Result as IOResult, path::Path};

/// Reads the font files a document's `file` fonts point to.
pub trait ReadFontFile {
    /// The bytes of the font file at `path`.
    fn read_font_file(&self, path: &Path) -> IOResult<Vec<u8>>;
}
