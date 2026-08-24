#[cfg(feature = "impl")]
use serde::de::DeserializeOwned;
use std::io::Result as IOResult;
#[cfg(feature = "impl")]
use std::io::{Error as IOError, ErrorKind};
#[cfg(feature = "impl")]
use tyt_injection::serde_json::{self, Value};

/// Abstracts JSON deserialization for preference types.
///
/// The `impl` feature provides a blanket implementation for every type
/// implementing `serde::de::DeserializeOwned`.
pub trait DeserializePrefs: Sized {
    /// Deserializes the `key` section from a config file's JSON bytes.
    fn deserialize_prefs(config_json: &[u8], key: &str) -> IOResult<Option<Self>>;
}

#[cfg(feature = "impl")]
impl<T: DeserializeOwned> DeserializePrefs for T {
    fn deserialize_prefs(config_json: &[u8], key: &str) -> IOResult<Option<Self>> {
        let value: Value = serde_json::from_slice(config_json)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        let Some(section) = value.get(key) else {
            return Ok(None);
        };

        serde_json::from_value(section.clone())
            .map(Some)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
    }
}
