#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Identifier of a voxel snapshot inside a
/// [`VMaxContentsVmaxbFile`](crate::VMaxContentsVmaxbFile) (`s.id`).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxSnapshotId {
    /// Chunk id (0-511); Morton-decodes to the 8x8x8 chunk-grid coordinate.
    pub c: u32,

    /// Session id.
    #[cfg_attr(feature = "serde", serde(default))]
    pub s: i64,

    /// Snapshot type (4 = checkpoint).
    #[cfg_attr(feature = "serde", serde(default))]
    pub t: i64,
}
