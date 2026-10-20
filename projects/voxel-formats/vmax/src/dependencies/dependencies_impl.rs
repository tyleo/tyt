use crate::{DecodeBase64, EncodeBase64};
use base64::{Engine, engine::general_purpose::STANDARD};

/// The dependencies the `impl` feature provides.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl EncodeBase64 for DependenciesImpl {
    fn encode_base64(&self, bytes: &[u8]) -> String {
        STANDARD.encode(bytes)
    }
}

impl DecodeBase64 for DependenciesImpl {
    fn decode_base64(&self, text: &str) -> Result<Vec<u8>, String> {
        STANDARD.decode(text).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{DecodeBase64, DependenciesImpl, EncodeBase64};

    #[test]
    fn round_trips_bytes() {
        let encoded = DependenciesImpl.encode_base64(&[0xC0, 0x00, 0xFF]);

        assert_eq!(encoded, "wAD/");
        assert_eq!(
            DependenciesImpl.decode_base64(&encoded),
            Ok(vec![0xC0, 0x00, 0xFF])
        );
    }

    /// `_` is a base64url character, outside the standard alphabet.
    #[test]
    fn rejects_base64url() {
        assert!(DependenciesImpl.decode_base64("w_==").is_err());
    }
}
