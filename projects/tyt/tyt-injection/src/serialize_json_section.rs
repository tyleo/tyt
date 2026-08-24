use serde_json::{Map, Value};
use std::io::{Error, ErrorKind, Result};

/// Builds the pretty-printed bytes of a JSON config file in which `key` maps
/// to a JSON encoding of `value`, and every other top-level section of
/// `existing` is preserved. Starts from an empty object when `existing` is
/// `None`.
pub fn serialize_json_section<T: serde::Serialize>(
    value: &T,
    key: &str,
    existing: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let mut root: Map<String, Value> = match existing {
        Some(bytes) => {
            let parsed: Value =
                serde_json::from_slice(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

            match parsed {
                Value::Object(map) => map,
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "config file root must be a JSON object",
                    ));
                }
            }
        }
        None => Map::new(),
    };

    let section = serde_json::to_value(value).map_err(Error::other)?;

    root.insert(key.to_string(), section);

    let mut bytes = serde_json::to_vec_pretty(&Value::Object(root)).map_err(Error::other)?;

    bytes.push(b'\n');

    Ok(bytes)
}
