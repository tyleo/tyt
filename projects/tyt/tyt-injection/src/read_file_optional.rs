use std::{
    fs,
    io::{ErrorKind, Result},
    path::Path,
};

/// Reads a file's contents, or returns `None` if the file does not exist.
pub fn read_file_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
