use crate::{Dependencies, DeserializePrefs, DirPrefs, OptionalDirPrefs, load_prefs_from_dir};
use std::io::{Error as IOError, Result as IOResult};

/// Loads the preference layer for `key` from `file_name` in the user home
/// directory.
///
/// Errors when the user home directory cannot be determined.
pub fn load_user_prefs<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    file_name: &str,
    key: &str,
) -> IOResult<OptionalDirPrefs<T>> {
    let Some(dir) = dependencies.user_home_dir()? else {
        return Err(IOError::other("user home directory cannot be determined"));
    };

    let prefs = load_prefs_from_dir(dependencies, &dir, file_name, key)?;

    Ok(DirPrefs { dir, prefs })
}
