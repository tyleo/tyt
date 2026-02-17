use crate::{Dependencies, DeserializePrefs, Prefs};
use std::io::Result as IOResult;

/// Loads preferences for the given key from all `.tytconfig` locations.
pub fn load_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    key: &str,
) -> IOResult<Prefs<T>> {
    let user = crate::load_user_prefs(dependencies, key)?;
    let git_root = crate::load_git_prefs(dependencies, key)?;
    Ok(Prefs { user, git_root })
}
