use std::{io::Result as IOResult, path::Path};

/// Creates directories.
pub trait CreateDirAll {
    /// Creates `path` and any missing parents. An existing directory passes.
    fn create_dir_all(&self, path: &Path) -> IOResult<()>;
}
