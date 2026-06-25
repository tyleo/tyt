use crate::{Dependencies, Error, Result};
use std::ffi::OsStr;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_injection::exec_map("magick", args, Error::IO, Error::Magick)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
