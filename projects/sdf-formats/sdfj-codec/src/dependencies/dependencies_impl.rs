use crate::{DecodeSdfjJson, Deflate, EncodeSdfjJson, Inflate};
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use sdfj::SdfjFile;
use std::io::{Read, Write};

/// The dependencies the `impl` feature provides.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl DecodeSdfjJson for DependenciesImpl {
    fn decode_sdfj_json(&self, bytes: &[u8]) -> Result<SdfjFile, String> {
        serde_json::from_slice(bytes).map_err(|error| error.to_string())
    }
}

impl EncodeSdfjJson for DependenciesImpl {
    fn encode_sdfj_json(&self, file: &SdfjFile) -> Result<Vec<u8>, String> {
        serde_json::to_vec(file).map_err(|error| error.to_string())
    }

    fn encode_sdfj_json_pretty(&self, file: &SdfjFile) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(file).map_err(|error| error.to_string())
    }
}

impl Deflate for DependenciesImpl {
    fn deflate(&self, bytes: &[u8]) -> Vec<u8> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(bytes)
            .expect("write to Vec is infallible");
        encoder.finish().expect("flush to Vec is infallible")
    }
}

impl Inflate for DependenciesImpl {
    fn inflate(&self, stream: &[u8]) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        DeflateDecoder::new(stream)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Deflate, DependenciesImpl, Inflate};

    #[test]
    fn deflate_round_trips_and_ignores_trailing_bytes() {
        let text = b"{\"version\":1}";
        let mut stream = DependenciesImpl.deflate(text);
        assert!(stream.len() < text.len() + 8);
        assert_eq!(DependenciesImpl.inflate(&stream).unwrap(), text);

        stream.extend_from_slice(b"PK\x01\x02trailing central directory");
        assert_eq!(DependenciesImpl.inflate(&stream).unwrap(), text);
    }

    #[test]
    fn inflate_rejects_a_malformed_stream() {
        assert!(DependenciesImpl.inflate(&[0xFF, 0xFF, 0xFF]).is_err());
    }
}
