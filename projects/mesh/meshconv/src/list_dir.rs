use crate::DirectoryEntry;
use std::{io::Result as IOResult, path::Path};

/// Lists a directory.
pub trait ListDir {
    /// The entries of the directory at `path`, in any order.
    fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>>;
}
