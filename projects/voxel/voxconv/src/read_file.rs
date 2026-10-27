use std::{io::Result as IOResult, path::Path};

/// Reads a whole file.
pub trait ReadFile {
    /// The bytes of the file at `path`.
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>>;
}
