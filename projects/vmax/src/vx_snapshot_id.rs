#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Identifier of a voxel snapshot inside a `VXObjectData` (`s.id`).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXSnapshotId {
    /// Chunk id (0–511); Morton-decodes to the 8×8×8 chunk-grid coordinate.
    pub c: u32,
    /// Session id (written only).
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub s: i64,
    /// Snapshot type, 4 = checkpoint (written only).
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub t: i64,
}
