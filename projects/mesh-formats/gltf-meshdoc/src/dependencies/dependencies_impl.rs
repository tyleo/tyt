use crate::{DecodeBase64, EncodeBase64};
use base64::{Engine, engine::general_purpose::STANDARD};

/// The dependencies over `base64`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl DecodeBase64 for DependenciesImpl {
    fn decode_base64(&self, text: &str) -> Result<Vec<u8>, String> {
        STANDARD.decode(text).map_err(|error| error.to_string())
    }
}

impl EncodeBase64 for DependenciesImpl {
    fn encode_base64(&self, bytes: &[u8]) -> String {
        STANDARD.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DecodeBase64, DependenciesImpl, EncodeBase64};

    #[test]
    fn base64_round_trips_and_pads() {
        assert_eq!(DependenciesImpl.encode_base64(&[0xC0]), "wA==");
        assert_eq!(DependenciesImpl.decode_base64("wA==").unwrap(), [0xC0]);
        assert!(DependenciesImpl.decode_base64("not base64!").is_err());
    }
}
