use crate::{DecodeVoxjJson, Deflate, EncodeVoxjJson, Inflate};
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use std::io::{Read, Write};
use voxj::{
    CostVoxjObject, DecodeBase64, DependenciesImpl as VoxjDependenciesImpl, EncodeBase64, VoxjFile,
    VoxjObject,
};

/// The dependencies over `serde_json` and `flate2`. It also implements the
/// `voxj` crate's traits over [`voxj::DependenciesImpl`], so one value serves
/// the whole family.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl DecodeVoxjJson for DependenciesImpl {
    fn decode_voxj_json(&self, bytes: &[u8]) -> Result<VoxjFile, String> {
        serde_json::from_slice(bytes).map_err(|error| error.to_string())
    }
}

impl EncodeVoxjJson for DependenciesImpl {
    fn encode_voxj_json(&self, file: &VoxjFile) -> Vec<u8> {
        serde_json::to_vec(file).expect("a document holds nothing without a JSON form")
    }

    fn encode_voxj_json_pretty(&self, file: &VoxjFile) -> Vec<u8> {
        serde_json::to_vec_pretty(file).expect("a document holds nothing without a JSON form")
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

impl EncodeBase64 for DependenciesImpl {
    fn encode_base64(&self, bytes: &[u8]) -> String {
        VoxjDependenciesImpl.encode_base64(bytes)
    }
}

impl DecodeBase64 for DependenciesImpl {
    fn decode_base64(&self, text: &str) -> Result<Vec<u8>, String> {
        VoxjDependenciesImpl.decode_base64(text)
    }
}

impl CostVoxjObject for DependenciesImpl {
    fn cost_voxj_object(&self, object: &VoxjObject) -> usize {
        VoxjDependenciesImpl.cost_voxj_object(object)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Deflate, DependenciesImpl, Inflate};

    #[test]
    fn deflate_round_trips_and_ignores_trailing_bytes() {
        let text = b"{\"version\":1,\"main\":{}}";
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
