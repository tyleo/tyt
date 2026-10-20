use crate::{DecodeSdfjJson, EncodeSdfjJson};
use sdfj::SdfjFile;

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
