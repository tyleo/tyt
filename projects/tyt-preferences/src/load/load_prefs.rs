use crate::{Dependencies, Prefs, load_git_prefs, load_user_prefs};
use serde::de::DeserializeOwned;
use std::io;

/// Loads preferences for the given key from all `.tytconfig` locations.
pub fn load_prefs<T: DeserializeOwned>(
    dependencies: &impl Dependencies,
    key: &str,
) -> io::Result<Prefs<T>> {
    let user = load_user_prefs(dependencies, key)?;
    let git_root = load_git_prefs(dependencies, key)?;
    Ok(Prefs { user, git_root })
}
