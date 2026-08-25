use crate::{Dependencies, DeserializePrefs, DirPrefs, OptionalDirPrefs, load_prefs_from_dir};
use std::io::{Error as IOError, Result as IOResult};

/// Loads the preference layer for `key` from `~/.tytconfig`.
///
/// Errors when the user home directory cannot be determined.
pub fn load_user_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    key: &str,
) -> IOResult<OptionalDirPrefs<T>> {
    let Some(dir) = dependencies.user_home_dir()? else {
        return Err(IOError::other("user home directory cannot be determined"));
    };

    let prefs = load_prefs_from_dir(dependencies, &dir, ".tytconfig", key)?;

    Ok(DirPrefs { dir, prefs })
}
