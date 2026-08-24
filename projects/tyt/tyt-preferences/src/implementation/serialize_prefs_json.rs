use serde::Serialize;
use serde_json::{Map, Value};
use std::io::{Error as IOError, ErrorKind, Result as IOResult};

/// Builds the pretty-printed file bytes in which `key` maps to a JSON
/// encoding of `value`, and every other top-level section of `existing` is
/// preserved. Starts from an empty object when `existing` is `None`. A
/// ready-made body for `SerializePrefs` impls.
pub fn serialize_prefs_json<T: Serialize>(
    value: &T,
    key: &str,
    existing: Option<&[u8]>,
) -> IOResult<Vec<u8>> {
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
