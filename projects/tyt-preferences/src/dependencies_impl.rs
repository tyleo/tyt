use crate::Dependencies;
use std::{
    env, fs,
    io::{Error as IOError, ErrorKind, Result as IOResult},
    path::{Path, PathBuf},
    process::Command,
};

/// Concrete implementation of preference I/O operations.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn user_home_dir(&self) -> IOResult<Option<PathBuf>> {
        Ok(env::var_os("HOME").map(PathBuf::from))
    }

    fn git_root_dir(&self) -> IOResult<Option<PathBuf>> {
        let output = match Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            Ok(output) => output,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        if !output.status.success() {
            return Ok(None);
        }

        let path = String::from_utf8(output.stdout)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(Some(PathBuf::from(path.trim())))
    }

    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}
