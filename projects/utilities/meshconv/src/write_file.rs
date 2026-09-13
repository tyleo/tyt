use std::{io::Result as IOResult, path::Path};

/// Writes a whole file.
pub trait WriteFile {
    /// Writes `bytes` to the file at `path`, replacing any file there and
    /// creating missing parent directories.
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()>;
}
