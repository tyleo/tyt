use crate::{Dependencies, DeserializePrefs};
use std::{io::Result as IOResult, path::Path};

/// Reads a single section keyed by `key` from a `.tytconfig` / `.tytusrconfig`
/// file at `path`. Returns `None` if the file does not exist or the section
/// is absent.
pub fn read_section<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    path: &Path,
    key: &str,
) -> IOResult<Option<T>> {
    let Some(bytes) = dependencies.read_file(path)? else {
        return Ok(None);
    };
    T::deserialize_prefs(&bytes, key)
}
