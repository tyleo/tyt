#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An encoded voxel-position block. The encoding fixes the object's voxel
/// order, which the sample channels follow. Serde renders this as
/// `{ "encoding", "data" }`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "encoding", content = "data"))]
pub enum VoxjPositionBlock {
    /// One `[x, y, z]` triple per voxel, in listing order.
    #[cfg_attr(feature = "serde", serde(rename = "raw-json"))]
    RawJson(Vec<[u32; 3]>),

    /// Dense base64 occupancy bitmap over the object's
    /// [`bounds`](crate::VoxjObject::bounds), in ascending cell-index order.
    #[cfg_attr(feature = "serde", serde(rename = "bitmap-base64"))]
    BitmapBase64(String),

    /// Base64 varint stream of prefix-sum deltas of each voxel's ascending 3D
    /// Hilbert index.
    #[cfg_attr(feature = "serde", serde(rename = "hilbert_index-delta-varint-base64"))]
    HilbertIndexDeltaVarintBase64(String),
}
