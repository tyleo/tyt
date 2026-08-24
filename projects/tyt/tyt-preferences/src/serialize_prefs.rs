#[cfg(feature = "impl")]
use serde::Serialize;
use std::io::Result as IOResult;
#[cfg(feature = "impl")]
use std::io::{Error as IOError, ErrorKind};
#[cfg(feature = "impl")]
use tyt_injection::serde_json::{self, Map, Value};

/// Abstracts JSON serialization for preference types.
///
/// The `impl` feature provides a blanket implementation for every type
/// implementing `serde::Serialize`.
pub trait SerializePrefs {
    /// Builds the pretty-printed file bytes to write back: `key` maps to a
    /// JSON encoding of `self`, and every other top-level section of
    /// `existing` is preserved. Starts from an empty object when `existing`
    /// is `None`.
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>>;
}

#[cfg(feature = "impl")]
impl<T: Serialize> SerializePrefs for T {
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>> {
        let mut root: Map<String, Value> = match existing {
            Some(bytes) => {
                let parsed: Value = serde_json::from_slice(bytes)
                    .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

                match parsed {
                    Value::Object(map) => map,
                    _ => {
                        return Err(IOError::new(
                            ErrorKind::InvalidData,
                            "config file root must be a JSON object",
                        ));
                    }
                }
            }
            None => Map::new(),
        };

        let value = serde_json::to_value(self).map_err(IOError::other)?;

        root.insert(key.to_string(), value);

        let mut bytes = serde_json::to_vec_pretty(&Value::Object(root)).map_err(IOError::other)?;

        bytes.push(b'\n');

        Ok(bytes)
    }
}
