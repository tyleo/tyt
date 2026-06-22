#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An encoded voxel-position block. The chosen encoding fixes the object's
/// canonical voxel order, which the sample channels then follow. Base64 strings
/// are stored already-encoded; serde renders this as `{ "encoding", "data" }`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "encoding", content = "data"))]
pub enum VoxjPositionBlock {
    /// One `[x, y, z]` triple per voxel, in listing order.
    #[cfg_attr(feature = "serde", serde(rename = "raw-json"))]
    RawJson(Vec<[u32; 3]>),

    /// Dense occupancy bitmap over [`bounds`](crate::VoxjObject::bounds),
    /// packed 8 bits per byte MSB-first, base64-encoded; canonical order is
    /// ascending cell index.
    #[cfg_attr(feature = "serde", serde(rename = "bitmap-base64"))]
    BitmapBase64(String),

    /// Prefix-sum deltas of each voxel's 3D Hilbert-curve index (ascending), as
    /// an unsigned-LEB128 varint stream, base64-encoded.
    #[cfg_attr(feature = "serde", serde(rename = "hilbert_index-delta-varint-base64"))]
    HilbertIndexDeltaVarintBase64(String),
}
