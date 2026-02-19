use std::io::{Error, ErrorKind, Result};

pub fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
