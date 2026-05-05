use crate::{Dependencies, SerializePrefs};
use std::{io::Result as IOResult, path::Path};

/// Writes `value` as the `key` section of a `.tytconfig` / `.tytusrconfig`
/// file at `path`, preserving all other top-level sections. Creates the file
/// if it does not exist. Pretty-prints the JSON.
pub fn write_section<T: SerializePrefs>(
    dependencies: &impl Dependencies,
    path: &Path,
    key: &str,
    value: &T,
) -> IOResult<()> {
    let existing = dependencies.read_file(path)?;
    let bytes = value.serialize_prefs(key, existing.as_deref())?;
    dependencies.write_file(path, &bytes)
}
