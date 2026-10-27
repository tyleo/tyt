use std::{io::Result as IOResult, path::Path};

/// Moves files and directories.
pub trait RenamePath {
    /// Moves `from` to `to` on the same filesystem.
    fn rename_path(&self, from: &Path, to: &Path) -> IOResult<()>;
}
