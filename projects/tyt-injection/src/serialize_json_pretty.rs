use std::io::{Error, ErrorKind, Result};

pub fn serialize_json_pretty<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
