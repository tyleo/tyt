use crate::{DependenciesImpl, EncodeBase64};

/// Standard base64 of `bytes`, for hand-built blocks.
pub fn standard_base64(bytes: &[u8]) -> String {
    DependenciesImpl.encode_base64(bytes)
}
