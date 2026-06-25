use std::io::{Error, ErrorKind, Result};

/// Deserializes a binary (or XML) property list into `T`.
pub fn parse_bplist<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    plist::from_bytes(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
