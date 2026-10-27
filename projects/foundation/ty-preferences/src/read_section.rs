use crate::{Dependencies, DeserializePrefs};
use std::{io::Result as IOResult, path::Path};

/// Reads the `key` section from the config file at `path`. Returns `None` if
/// the file does not exist or the section is absent.
pub fn read_section<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    path: &Path,
    key: &str,
) -> IOResult<Option<T>> {
    let Some(bytes) = dependencies.read_file(path)? else {
        return Ok(None);
    };

    codec.deserialize_prefs(&bytes, key)
}
