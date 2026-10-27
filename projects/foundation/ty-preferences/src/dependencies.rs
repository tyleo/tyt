use std::{io::Result as IOResult, path::Path};

/// Dependency injection for preference file I/O.
pub trait Dependencies {
    /// Reads the contents of a file, or returns `None` if the file does not
    /// exist.
    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>>;

    /// Writes the given bytes to `path`, atomically replacing any existing
    /// file.
    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()>;
}
