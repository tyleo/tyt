use crate::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub trait Dependencies {
    fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()>;

    fn create_temp_dir(&self) -> Result<PathBuf>;

    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn glob_single_match(&self, pattern: &str) -> Result<PathBuf>;

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
