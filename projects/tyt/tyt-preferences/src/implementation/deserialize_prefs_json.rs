use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io::{Error as IOError, ErrorKind, Result as IOResult};

/// Deserializes the `key` section from a config file's JSON bytes. Returns
/// `None` if the section is absent. A ready-made body for `DeserializePrefs`
/// impls.
pub fn deserialize_prefs_json<T: DeserializeOwned>(
    config_json: &[u8],
    key: &str,
) -> IOResult<Option<T>> {
    let value: Value =
        serde_json::from_slice(config_json).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    let Some(section) = value.get(key) else {
        return Ok(None);
    };

    serde_json::from_value(section.clone())
        .map(Some)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}
