use crate::{Dependencies, SerializePrefs};
use std::{io::Result as IOResult, path::Path};

/// Writes `value` as the `key` section of the config file at `path`, preserving
/// all other top-level sections. Creates the file if it does not exist.
pub fn write_section<T>(
    dependencies: &impl Dependencies,
    codec: &impl SerializePrefs<T>,
    path: &Path,
    key: &str,
    value: &T,
) -> IOResult<()> {
    let existing = dependencies.read_file(path)?;

    let bytes = codec.serialize_prefs(value, key, existing.as_deref())?;

    dependencies.write_file(path, &bytes)
}
