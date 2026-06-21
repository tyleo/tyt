use crate::{VXSnapshotId, VXStats};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel snapshot storage holding one chunk's dense voxel byte stream.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXStorage {
    pub id: VXSnapshotId,
    /// Dense voxel bytes: two per slot (material, color), indexed by Morton
    /// code offset from `st.min[3]`.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub ds: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub st: VXStats,
    /// Layer-color usage mask (256 bytes).
    #[cfg_attr(feature = "serde", serde(default, with = "serde_bytes"))]
    pub lc: Vec<u8>,
    /// Deleted layer-color usage mask (256 bytes).
    #[cfg_attr(feature = "serde", serde(default, with = "serde_bytes"))]
    pub dlc: Vec<u8>,
}
