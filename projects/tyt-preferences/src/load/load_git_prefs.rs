use crate::{Dependencies, load_prefs_from_dir};
use serde::de::DeserializeOwned;
use std::io;

/// Loads preferences for the given key from `<git-root>/.tytconfig`.
pub fn load_git_prefs<T: DeserializeOwned>(
    dependencies: &impl Dependencies,
    key: &str,
) -> io::Result<Option<T>> {
    let Some(dir) = dependencies.git_root_dir()? else {
        return Ok(None);
    };
    load_prefs_from_dir(dependencies, &dir, key)
}
