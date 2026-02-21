use std::io::{self, Error as IOError, ErrorKind};

/// Abstracts JSON deserialization for preference types.
///
/// When the `impl` feature is enabled, a blanket implementation is provided
/// for all types implementing `serde::de::DeserializeOwned`.
pub trait DeserializePrefs: Sized {
    /// Deserializes a preference value from a config file's raw JSON bytes and key.
    fn deserialize_prefs(config_json: &[u8], key: &str) -> io::Result<Option<Self>>;
}

#[cfg(feature = "impl")]
impl<T: serde::de::DeserializeOwned> DeserializePrefs for T {
    fn deserialize_prefs(config_json: &[u8], key: &str) -> io::Result<Option<Self>> {
        let value: serde_json::Value = serde_json::from_slice(config_json)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        let Some(section) = value.get(key) else {
            return Ok(None);
        };
        serde_json::from_value(section.clone())
            .map(Some)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
    }
}
