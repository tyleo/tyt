use crate::{VMaxSnapshotId, VMaxStats};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel snapshot storage holding one chunk's dense voxel byte stream.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxStorage {
    pub id: VMaxSnapshotId,

    /// Dense voxel bytes: two per slot (material, color), indexed by Morton
    /// code offset from `st.min[3]`.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub ds: Vec<u8>,

    #[cfg_attr(feature = "serde", serde(default))]
    pub st: VMaxStats,

    /// Layer-color usage mask (256 bytes).
    #[cfg_attr(feature = "serde", serde(default, with = "serde_bytes"))]
    pub lc: Vec<u8>,

    /// Deleted layer-color usage mask (256 bytes).
    #[cfg_attr(feature = "serde", serde(default, with = "serde_bytes"))]
    pub dlc: Vec<u8>,

    /// Legacy usage mask: older files carry `is` in place of `lc`/`dlc`.
    #[cfg_attr(
        feature = "serde",
        serde(default, with = "serde_bytes", skip_serializing_if = "Vec::is_empty")
    )]
    pub is: Vec<u8>,
}
