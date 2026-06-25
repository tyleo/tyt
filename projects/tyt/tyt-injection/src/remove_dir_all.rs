use std::{
    fs,
    io::{ErrorKind, Result},
    path::Path,
};

pub fn remove_dir_all(path: &Path) -> Result<()> {
    // If it's already gone, treat as success.
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
