use serde::Deserialize;

/// Identifier of a voxel snapshot inside a `VXObjectData` (`s.id`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct VXSnapshotIdSerde {
    /// Chunk id (0–511); Morton-decodes to the 8×8×8 chunk-grid coordinate.
    pub c: u32,
}
