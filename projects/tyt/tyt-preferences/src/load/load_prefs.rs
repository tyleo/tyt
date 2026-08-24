use crate::{Dependencies, DeserializePrefs, Prefs, load_git_prefs, load_user_prefs};
use std::io::Result as IOResult;

/// Loads preferences for `key` from all `.tytconfig` locations.
pub fn load_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    key: &str,
) -> IOResult<Prefs<T>> {
    let user = load_user_prefs(dependencies, key)?;
    let git_root = load_git_prefs(dependencies, key)?;
    Ok(Prefs { user, git_root })
}
