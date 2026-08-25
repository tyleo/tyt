use crate::{Dependencies, DeserializePrefs, DirPrefs, OptionalDirPrefs, load_prefs_from_dir};
use std::io::Result as IOResult;

/// Loads the preference layer for `key` from `<git-root>/.tytusrconfig`.
///
/// `.tytusrconfig` is the user-local sibling of `.tytconfig` for values that
/// should not be checked into the repo. Returns `None` outside a git
/// repository.
pub fn load_user_git_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    key: &str,
) -> IOResult<Option<OptionalDirPrefs<T>>> {
    let Some(dir) = dependencies.git_root_dir()? else {
        return Ok(None);
    };

    let prefs = load_prefs_from_dir(dependencies, &dir, ".tytusrconfig", key)?;

    Ok(Some(DirPrefs { dir, prefs }))
}
