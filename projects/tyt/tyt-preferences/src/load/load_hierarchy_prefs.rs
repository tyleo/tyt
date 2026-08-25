use crate::{Dependencies, DeserializePrefs, DirPrefs, load_prefs_from_dir};
use std::io::{Error as IOError, Result as IOResult};

/// Loads preference layers for `key` from the `file_name` in every directory
/// from the git root down to cwd, furthest from cwd first. Directories that
/// supply no prefs are omitted.
///
/// Returns an empty list outside a git repository. Errors when cwd is not
/// under the git root.
pub fn load_hierarchy_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    file_name: &str,
    key: &str,
) -> IOResult<Vec<DirPrefs<T>>> {
    let Some(root) = dependencies.git_root_dir()? else {
        return Ok(Vec::new());
    };

    let cwd = dependencies.current_dir()?;

    let mut dirs = Vec::new();

    for dir in cwd.ancestors() {
        dirs.push(dir);

        if dir == root {
            break;
        }
    }

    if dirs.last().copied() != Some(root.as_path()) {
        return Err(IOError::other(format!(
            "current directory {} is not under the git root {}",
            cwd.display(),
            root.display()
        )));
    }

    let mut layers = Vec::new();

    for dir in dirs.into_iter().rev() {
        let Some(prefs) = load_prefs_from_dir(dependencies, dir, file_name, key)? else {
            continue;
        };

        layers.push(DirPrefs {
            dir: dir.to_path_buf(),
            prefs,
        });
    }

    Ok(layers)
}
