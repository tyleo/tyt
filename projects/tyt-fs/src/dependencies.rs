use crate::{Prefs, Result};
use std::{ffi::OsStr, path::Path};

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Creates all missing parent directories for the given path.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Executes `rg` with the given arguments and returns stdout.
    fn exec_rg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Returns the preferences for this crate.
    fn fs_prefs(&self) -> Result<Prefs>;

    /// Moves a file from one path to another.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Writes bytes to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
