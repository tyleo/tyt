use crate::{Dependencies, DeserializePrefs, DirPrefs, OptionalDirPrefs, load_prefs_from_dir};
use std::io::Result as IOResult;

/// Loads the preference layer for `key` from `file_name` in the git root.
///
/// Returns `None` outside a git repository.
pub fn load_git_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    file_name: &str,
    key: &str,
) -> IOResult<Option<OptionalDirPrefs<T>>> {
    let Some(dir) = dependencies.git_root_dir()? else {
        return Ok(None);
    };

    let prefs = load_prefs_from_dir(dependencies, &dir, file_name, key)?;

    Ok(Some(DirPrefs { dir, prefs }))
}
