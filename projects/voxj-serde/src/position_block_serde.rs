use serde::{Deserialize, Serialize};
use voxj::PositionBlock;

/// Serde-compatible parity type for [`PositionBlock`], serialized as
/// `{ "encoding": ..., "data": ... }`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "encoding", content = "data")]
pub enum PositionBlockSerde {
    #[serde(rename = "raw-json")]
    RawJson(Vec<[u32; 3]>),
    #[serde(rename = "bitmap-base64")]
    BitmapBase64(String),
    #[serde(rename = "hilbert_index-delta-varint-base64")]
    HilbertIndexDeltaVarintBase64(String),
}

impl From<PositionBlock> for PositionBlockSerde {
    fn from(v: PositionBlock) -> Self {
        match v {
            PositionBlock::RawJson(p) => PositionBlockSerde::RawJson(p),
            PositionBlock::BitmapBase64(s) => PositionBlockSerde::BitmapBase64(s),
            PositionBlock::HilbertIndexDeltaVarintBase64(s) => {
                PositionBlockSerde::HilbertIndexDeltaVarintBase64(s)
            }
        }
    }
}

impl From<PositionBlockSerde> for PositionBlock {
    fn from(v: PositionBlockSerde) -> Self {
        match v {
            PositionBlockSerde::RawJson(p) => PositionBlock::RawJson(p),
            PositionBlockSerde::BitmapBase64(s) => PositionBlock::BitmapBase64(s),
            PositionBlockSerde::HilbertIndexDeltaVarintBase64(s) => {
                PositionBlock::HilbertIndexDeltaVarintBase64(s)
            }
        }
    }
}
