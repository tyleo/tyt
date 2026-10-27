use std::{ffi::OsString, io::Result as IOResult, path::Path};

/// The side effects the completion commands perform.
pub trait Dependencies {
    /// The value of the environment variable `name`, or `None` when unset.
    fn read_env_var(&self, name: &str) -> Option<OsString>;

    /// Writes `bytes` to `path`, creating its missing parents.
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()>;

    /// Writes `bytes` to standard output.
    fn write_stdout(&self, bytes: &[u8]) -> IOResult<()>;
}
