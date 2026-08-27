use crate::{Dependencies, DeserializePrefs, DirPrefs, load_sources_prefs};
use std::{
    io::{Error as IOError, Result as IOResult},
    path::Path,
};

/// Loads preference layers for `key` from the `file_name` in every directory
/// from `git_root` down to `cwd`, furthest from `cwd` first. Directories that
/// supply no prefs are omitted.
///
/// Errors when `cwd` is not under `git_root`.
pub fn load_hierarchy_prefs<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    git_root: &Path,
    cwd: &Path,
    file_name: &str,
    key: &str,
) -> IOResult<Vec<DirPrefs<T>>> {
    let mut dirs = Vec::new();

    for dir in cwd.ancestors() {
        dirs.push(dir);

        if dir == git_root {
            break;
        }
    }

    if dirs.last().copied() != Some(git_root) {
        return Err(IOError::other(format!(
            "current directory {} is not under the git root {}",
            cwd.display(),
            git_root.display()
        )));
    }

    let sources: Vec<(&Path, &str)> = dirs.into_iter().rev().map(|dir| (dir, file_name)).collect();

    load_sources_prefs(dependencies, codec, &sources, key)
}
