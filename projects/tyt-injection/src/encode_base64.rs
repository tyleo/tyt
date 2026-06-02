use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Encodes bytes as a standard (RFC 4648) base64 string.
pub fn encode_base64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}
