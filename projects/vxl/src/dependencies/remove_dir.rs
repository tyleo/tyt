use std::{io::Result as IOResult, path::Path};

/// Removes directories.
pub trait RemoveDir {
    /// Removes the empty directory at `path`. Fails on a directory with
    /// contents.
    fn remove_dir(&self, path: &Path) -> IOResult<()>;
}
