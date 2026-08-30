#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An encoded voxel-sample block, in the position block's voxel order. Serde
/// renders this as `{ "encoding", "data" }`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "encoding", content = "data", deny_unknown_fields)
)]
pub enum VoxjSampleBlock {
    /// One channel per layer: that layer's material index for every voxel, in
    /// order.
    #[cfg_attr(feature = "serde", serde(rename = "raw-json"))]
    RawJson(Vec<Vec<u32>>),

    /// One run-length channel per layer: a flat `[value, count, ...]` stream.
    #[cfg_attr(feature = "serde", serde(rename = "rle-json"))]
    RleJson(Vec<Vec<u32>>),

    /// One base64 bit-packed channel per layer.
    #[cfg_attr(feature = "serde", serde(rename = "packed-base64"))]
    PackedBase64(Vec<String>),
}
