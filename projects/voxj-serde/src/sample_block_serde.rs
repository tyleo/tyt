use serde::{Deserialize, Serialize};
use voxj::SampleBlock;

/// Serde-compatible parity type for [`SampleBlock`], serialized as
/// `{ "encoding": ..., "data": ... }`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "encoding", content = "data")]
pub enum SampleBlockSerde {
    #[serde(rename = "raw-json")]
    RawJson(Vec<Vec<u32>>),
    #[serde(rename = "rle-json")]
    RleJson(Vec<Vec<u32>>),
    #[serde(rename = "packed-base64")]
    PackedBase64(Vec<String>),
}

impl From<SampleBlock> for SampleBlockSerde {
    fn from(v: SampleBlock) -> Self {
        match v {
            SampleBlock::RawJson(d) => SampleBlockSerde::RawJson(d),
            SampleBlock::RleJson(d) => SampleBlockSerde::RleJson(d),
            SampleBlock::PackedBase64(d) => SampleBlockSerde::PackedBase64(d),
        }
    }
}

impl From<SampleBlockSerde> for SampleBlock {
    fn from(v: SampleBlockSerde) -> Self {
        match v {
            SampleBlockSerde::RawJson(d) => SampleBlock::RawJson(d),
            SampleBlockSerde::RleJson(d) => SampleBlock::RleJson(d),
            SampleBlockSerde::PackedBase64(d) => SampleBlock::PackedBase64(d),
        }
    }
}
