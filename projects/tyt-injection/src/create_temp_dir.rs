use crate::unique_temp_path;
use std::{
    fs,
    io::{Error as IOError, ErrorKind, Result},
    path::PathBuf,
};

pub fn create_temp_dir() -> Result<PathBuf> {
    // Try a handful of times in the unlikely event of a name collision.
    for _ in 0..16 {
        let path = unique_temp_path()?;
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }

    Err(IOError::new(
        ErrorKind::AlreadyExists,
        "failed to create a unique temp dir after multiple attempts",
    ))
}
