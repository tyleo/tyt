use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::io::{Error, ErrorKind, Result};

/// Decodes a standard (RFC 4648) base64 string into bytes.
pub fn decode_base64(text: &str) -> Result<Vec<u8>> {
    STANDARD
        .decode(text)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
