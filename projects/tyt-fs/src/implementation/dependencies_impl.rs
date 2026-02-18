use crate::{Dependencies, Error, Prefs, Result};
use std::{
    ffi::OsStr,
    fs::{self},
    path::{Path, PathBuf},
};
use tyt_preferences::Dependencies as _;

/// Concrete implementation of filesystem operations.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(fs::create_dir_all(path)?)
    }

    fn exec_rg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_injection::exec_map("rg", args, Error::IO, Error::Rg)
    }

    fn fs_prefs(&self) -> Result<Prefs> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let mut prefs: Prefs = tyt_preferences::load_git_prefs(&prefs_deps, "fs")
            .map_err(Error::IO)?
            .unwrap_or_default();
        if let Some(ref scratch_dir) = prefs.scratch_dir {
            let scratch_path = PathBuf::from(scratch_dir);
            if !scratch_path.is_absolute()
                && let Some(git_root) = prefs_deps.git_root_dir().map_err(Error::IO)?
            {
                prefs.scratch_dir =
                    Some(git_root.join(scratch_path).to_string_lossy().into_owned());
            }
        }
        Ok(prefs)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(fs::rename(from, to)?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
