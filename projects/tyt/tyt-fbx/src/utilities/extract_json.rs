use crate::Result;
use std::io::{Error as IOError, ErrorKind};

/// Extracts a JSON value from bytes by finding the first `open` and last `close` delimiter.
pub fn extract_json(bytes: &[u8], open: u8, close: u8) -> Result<&[u8]> {
    let start = bytes.iter().position(|&b| b == open).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("no '{}' found", open as char),
        )
    })?;
    let end = bytes.iter().rposition(|&b| b == close).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("no '{}' found", close as char),
        )
    })?;
    Ok(&bytes[start..=end])
}
