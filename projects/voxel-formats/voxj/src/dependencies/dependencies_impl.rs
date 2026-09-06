use crate::{CostVoxjObject, DecodeBase64, EncodeBase64, VoxjObject};
use base64::{Engine, engine::general_purpose::STANDARD};
use flate2::{Compression, write::DeflateEncoder};
use std::io::Write;

/// The dependencies over the `base64` crate's standard engine and a deflate
/// cost: an object costs the bytes its compact JSON takes deflated.
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

impl CostVoxjObject for DependenciesImpl {
    fn cost_voxj_object(&self, object: &VoxjObject) -> usize {
        let json = serde_json::to_vec(object).expect("an object holds nothing without a JSON form");
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&json)
            .expect("write to Vec is infallible");
        encoder.finish().expect("flush to Vec is infallible").len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CostVoxjObject, DecodeBase64, DependenciesImpl, EncodeBase64, VoxjObject,
        VoxjPositionBlock, VoxjSampleBlock,
    };

    #[test]
    fn round_trips_with_padding() {
        let encoded = DependenciesImpl.encode_base64(&[0xC0]);
        assert_eq!(encoded, "wA==");
        assert_eq!(DependenciesImpl.decode_base64(&encoded), Ok(vec![0xC0]));
    }

    /// `_` is a base64url character, outside the standard alphabet.
    #[test]
    fn rejects_the_url_alphabet() {
        assert!(DependenciesImpl.decode_base64("w_==").is_err());
    }

    /// A one-layer object with `voxels` raw voxels along x.
    fn object(voxels: u32) -> VoxjObject {
        VoxjObject {
            name: "o".to_owned(),
            layers: vec![0],
            bounds: [voxels, 1, 1],
            origin: [0, 0, 0],
            voxel_positions: VoxjPositionBlock::RawJson((0..voxels).map(|x| [x, 0, 0]).collect()),
            voxel_samples: VoxjSampleBlock::RawJson(vec![(0..voxels).collect()]),
        }
    }

    #[test]
    fn cost_grows_with_the_object_and_stays_under_the_json() {
        let small = DependenciesImpl.cost_voxj_object(&object(2));
        let large = DependenciesImpl.cost_voxj_object(&object(2000));
        assert!(small < large);
        assert!(large < serde_json::to_vec(&object(2000)).unwrap().len());
    }
}
