use crate::{Dependencies, Error, FsPrefs, Result};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tyt_preferences::Dependencies as _;

/// Concrete implementation of filesystem operations.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(std::fs::create_dir_all(path)?)
    }

    fn exec_rg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_injection::exec_map("rg", args, Error::IO, Error::Rg)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(std::fs::rename(from, to)?)
    }

    fn scratch_dir(&self) -> Result<Option<PathBuf>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let prefs: Option<FsPrefs> =
            tyt_preferences::load_git_prefs(&prefs_deps, "fs").map_err(Error::IO)?;
        let Some(scratch_dir) = prefs.and_then(|p| p.scratch_dir) else {
            return Ok(None);
        };
        let scratch_path = PathBuf::from(&scratch_dir);
        if scratch_path.is_absolute() {
            return Ok(Some(scratch_path));
        }
        let git_root = prefs_deps.git_root_dir().map_err(Error::IO)?;
        Ok(git_root.map(|root| root.join(scratch_path)))
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
