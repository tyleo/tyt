use crate::{Dependencies, DeserializePrefs};
use std::{io::Result as IOResult, path::Path};

const CONFIG_FILE_NAME: &str = ".tytconfig";

pub(crate) fn load_prefs_from_dir<T: DeserializePrefs>(
    dependencies: &impl Dependencies,
    dir: &Path,
    key: &str,
) -> IOResult<Option<T>> {
    let path = dir.join(CONFIG_FILE_NAME);
    let Some(bytes) = dependencies.read_file(&path)? else {
        return Ok(None);
    };
    T::deserialize_prefs(&bytes, key)
}
