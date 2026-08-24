use crate::{Dependencies, DeserializePrefs, load_prefs_from_dir};
use std::io::Result as IOResult;

/// Loads preferences for `key` from `<git-root>/.tytconfig`.
pub fn load_git_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    key: &str,
) -> IOResult<Option<T>> {
    let Some(dir) = dependencies.git_root_dir()? else {
        return Ok(None);
    };

    load_prefs_from_dir(dependencies, &dir, ".tytconfig", key)
}
