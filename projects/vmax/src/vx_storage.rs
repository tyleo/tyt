use crate::{VXSnapshotId, VXStats};
use serde::{Deserialize, Serialize};

/// Voxel snapshot storage (`s`) holding one chunk's dense voxel byte stream.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VXStorage {
    pub id: VXSnapshotId,
    /// Dense voxel bytes: two per slot (material, color), indexed by Morton code
    /// offset from `st.min[3]`.
    #[serde(with = "serde_bytes")]
    pub ds: Vec<u8>,
    #[serde(default)]
    pub st: VXStats,
    /// Layer-color usage mask, written only (256 bytes).
    #[serde(skip_deserializing, serialize_with = "serde_bytes::serialize")]
    pub lc: Vec<u8>,
    /// Deleted layer-color usage mask, written only (256 bytes).
    #[serde(skip_deserializing, serialize_with = "serde_bytes::serialize")]
    pub dlc: Vec<u8>,
}
