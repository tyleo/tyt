use crate::Result;
use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use voxsmith::{VMaxVoxMain, from_vmax_package};

/// Loads the whole `.vmax` package directory at `input` into a
/// [`VMaxVoxMain`]. The listing covers every package-relative path,
/// descending one level into subdirectories (only `QuickLook/`) so its
/// thumbnails keep their prefix. The resolver reads each path's bytes.
pub fn load_vmax_package(input: &Path) -> Result<VMaxVoxMain> {
    Ok(from_vmax_package(
        || {
            let mut paths = Vec::new();
            for entry in list_dir(input)? {
                let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if entry.is_dir() {
                    for child in list_dir(&entry)? {
                        if let Some(child) = child.file_name().and_then(|n| n.to_str()) {
                            paths.push(format!("{name}/{child}"));
                        }
                    }
                } else {
                    paths.push(name.to_owned());
                }
            }
            Ok(paths)
        },
        |name| match fs::read(input.join(name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        },
    )?)
}

/// Returns the entries of `path`, sorted by path.
fn list_dir(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(path)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();
    Ok(paths)
}
