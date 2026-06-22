#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An encoded voxel-sample block, in the position block's voxel order. Each
/// voxel carries one cell index per referenced palette. Serde renders this as
/// `{ "encoding", "data" }`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "encoding", content = "data"))]
pub enum VoxjSerdeSampleBlock {
    /// One row per voxel: that voxel's cell index per palette, in order.
    #[cfg_attr(feature = "serde", serde(rename = "raw-json"))]
    RawJson(Vec<Vec<u32>>),

    /// One channel per palette: a flat run stream `[value1, count1, ...]`.
    #[cfg_attr(feature = "serde", serde(rename = "rle-json"))]
    RleJson(Vec<Vec<u32>>),

    /// One channel per palette: each voxel's cell index bit-packed at width
    /// `max(1, bitLength(cellCount - 1))`, MSB-first, base64-encoded.
    #[cfg_attr(feature = "serde", serde(rename = "packed-base64"))]
    PackedBase64(Vec<String>),
}
