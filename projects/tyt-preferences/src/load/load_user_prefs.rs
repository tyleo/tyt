use crate::{Dependencies, DeserializePrefs, load::load_prefs_from_dir};
use std::io::Result as IOResult;

/// Loads preferences for the given key from `~/.tytconfig`.
pub fn load_user_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    key: &str,
) -> IOResult<Option<T>> {
    let Some(dir) = dependencies.user_home_dir()? else {
        return Ok(None);
    };
    load_prefs_from_dir(dependencies, &dir, key)
}
