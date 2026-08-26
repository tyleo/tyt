use crate::{DeserializePrefs, SerializePrefs};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use std::io::{Error as IOError, ErrorKind, Result as IOResult};

/// Strict JSON codec. Writing rebuilds the file as pretty-printed JSON.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonCodec;

impl<T: DeserializeOwned> DeserializePrefs<T> for JsonCodec {
    fn deserialize_prefs(&self, config: &[u8], key: &str) -> IOResult<Option<T>> {
        let value: Value =
            serde_json::from_slice(config).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        let Some(section) = value.get(key) else {
            return Ok(None);
        };

        serde_json::from_value(section.clone())
            .map(Some)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
    }
}

impl<T: Serialize> SerializePrefs<T> for JsonCodec {
    fn serialize_prefs(&self, value: &T, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>> {
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

        let section = serde_json::to_value(value).map_err(IOError::other)?;

        root.insert(key.to_string(), section);

        let mut bytes = serde_json::to_vec_pretty(&Value::Object(root)).map_err(IOError::other)?;

        bytes.push(b'\n');

        Ok(bytes)
    }
}
