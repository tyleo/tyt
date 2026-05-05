use crate::{Dependencies, Error, Result};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()> {
        fs::copy(from.as_ref(), to.as_ref())?;
        Ok(())
    }

    fn create_temp_dir(&self) -> Result<PathBuf> {
        Ok(tyt_injection::create_temp_dir()?)
    }

    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_injection::exec_map("magick", args, Error::IO, Error::Magick)
    }

    fn glob_single_match(&self, pattern: &str) -> Result<PathBuf> {
        tyt_injection::glob_single_match(pattern).map_err(|e| Error::Glob(format!("{e}")))
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(tyt_injection::remove_dir_all(path.as_ref())?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
