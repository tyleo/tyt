use crate::parse_json;
use std::io::{Error, ErrorKind, Result};

/// Deserializes the `key` section of a JSON config file's bytes. Returns
/// `None` if the section is absent.
pub fn parse_json_section<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    key: &str,
) -> Result<Option<T>> {
    let value: serde_json::Value = parse_json(bytes)?;

    let Some(section) = value.get(key) else {
        return Ok(None);
    };

    serde_json::from_value(section.clone())
        .map(Some)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
