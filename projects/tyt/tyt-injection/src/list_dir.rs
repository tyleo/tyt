use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

/// Returns sorted file paths from a directory listing.
pub fn list_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(path)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();
    Ok(paths)
}
