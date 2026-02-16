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
        let mut matches = Vec::new();
        for entry in glob::glob(pattern)
            .map_err(|e| Error::Glob(format!("invalid glob pattern '{pattern}': {e}")))?
        {
            matches
                .push(entry.map_err(|e| Error::Glob(format!("error reading glob result: {e}")))?);
        }

        match matches.len() {
            0 => Err(Error::Glob(format!("missing file matching: {pattern}"))),
            1 => Ok(matches.into_iter().next().unwrap()),
            n => {
                let mut msg = format!("multiple files ({n}) match '{pattern}':");
                for f in &matches {
                    msg.push_str(&format!("\n  {}", f.display()));
                }
                Err(Error::Glob(msg))
            }
        }
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(tyt_injection::remove_dir_all(path.as_ref())?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
