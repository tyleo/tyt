use crate::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Creates all missing parent directories for the given path.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Executes `rg` with the given arguments and returns stdout.
    fn exec_rg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Moves a file from one path to another.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Returns the configured scratch directory, or `None` if not configured.
    fn scratch_dir(&self) -> Result<Option<PathBuf>>;

    /// Writes bytes to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
