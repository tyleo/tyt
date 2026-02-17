use crate::{Dependencies, load_prefs_from_dir};
use serde::de::DeserializeOwned;
use std::io;

/// Loads preferences for the given key from `~/.tytconfig`.
pub fn load_user_prefs<T: DeserializeOwned>(
    dependencies: &impl Dependencies,
    key: &str,
) -> io::Result<Option<T>> {
    let Some(dir) = dependencies.user_home_dir()? else {
        return Ok(None);
    };
    load_prefs_from_dir(dependencies, &dir, key)
}
