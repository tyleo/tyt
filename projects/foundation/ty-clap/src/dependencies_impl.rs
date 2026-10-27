use crate::Dependencies;
use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, Result as IOResult, Write},
    path::Path,
};

/// The dependencies over the real environment, filesystem, and standard output.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn read_env_var(&self, name: &str) -> Option<OsString> {
        env::var_os(name)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, bytes)
    }

    fn write_stdout(&self, bytes: &[u8]) -> IOResult<()> {
        io::stdout().write_all(bytes)
    }
}
