use crate::{Dependencies, Error, Result};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

/// Concrete implementation of [`Dependencies`].
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_temp_dir(&self) -> Result<PathBuf> {
        Ok(tyt_common::create_temp_dir()?)
    }

    fn exec_ffmpeg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_common::exec("ffmpeg", args).map_err(|e| match e {
            tyt_common::ExecError::IO(e) => Error::IO(e),
            tyt_common::ExecError::Failed {
                exit_code,
                stdout,
                stderr,
            } => Error::Ffmpeg {
                exit_code,
                stdout,
                stderr,
            },
        })
    }

    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_common::exec("magick", args).map_err(|e| match e {
            tyt_common::ExecError::IO(e) => Error::IO(e),
            tyt_common::ExecError::Failed {
                exit_code,
                stdout,
                stderr,
            } => Error::Magick {
                exit_code,
                stdout,
                stderr,
            },
        })
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(tyt_common::remove_dir_all(path.as_ref())?)
    }

    fn rename_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()> {
        Ok(fs::rename(from.as_ref(), to.as_ref())?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_common::write_stdout(contents)?)
    }
}
