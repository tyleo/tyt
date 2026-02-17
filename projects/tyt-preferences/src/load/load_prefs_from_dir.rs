use crate::Dependencies;
use serde::de::DeserializeOwned;
use std::{
    io::{self, ErrorKind},
    path::Path,
};

const CONFIG_FILE_NAME: &str = ".tytconfig";

pub(crate) fn load_prefs_from_dir<T: DeserializeOwned>(
    dependencies: &impl Dependencies,
    dir: &Path,
    key: &str,
) -> io::Result<Option<T>> {
    let path = dir.join(CONFIG_FILE_NAME);
    let Some(bytes) = dependencies.read_file(&path)? else {
        return Ok(None);
    };
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    let Some(section) = value.get(key) else {
        return Ok(None);
    };
    serde_json::from_value(section.clone())
        .map(Some)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}
