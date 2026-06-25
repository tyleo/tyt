use std::io::{self, Error as IOError, ErrorKind};

/// Abstracts JSON serialization for preference types.
///
/// Given the existing file bytes (if any) and a section value, produces the
/// new pretty-printed bytes to write back, replacing only `key` and
/// preserving every other top-level section.
///
/// When the `impl` feature is enabled, a blanket implementation is provided
/// for all types implementing `serde::Serialize`.
pub trait SerializePrefs {
    /// Builds the file bytes to write so that `key` maps to a JSON encoding
    /// of `self`, merged into `existing` (or a fresh object if `None`).
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> io::Result<Vec<u8>>;
}

#[cfg(feature = "impl")]
impl<T: serde::Serialize> SerializePrefs for T {
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> io::Result<Vec<u8>> {
        use tyt_injection::serde_json::{self, Map, Value};
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
