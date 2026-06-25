use crate::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Dependency injection trait for cubemap operations.
pub trait Dependencies {
    fn create_temp_dir(&self) -> Result<PathBuf>;

    fn exec_ffmpeg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn rename_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()>;

    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
