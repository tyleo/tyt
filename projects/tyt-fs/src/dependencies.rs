use crate::Result;
use std::ffi::OsStr;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn exec_rg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
