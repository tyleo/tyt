use crate::{Dependencies, DeserializePrefs};
use std::{io::Result as IOResult, path::Path};

pub(crate) fn load_prefs_from_dir<T>(
    dependencies: &impl Dependencies,
    codec: &impl DeserializePrefs<T>,
    dir: &Path,
    file_name: &str,
    key: &str,
) -> IOResult<Option<T>> {
    let path = dir.join(file_name);

    let Some(bytes) = dependencies.read_file(&path)? else {
        return Ok(None);
    };

    codec.deserialize_prefs(&bytes, key)
}
