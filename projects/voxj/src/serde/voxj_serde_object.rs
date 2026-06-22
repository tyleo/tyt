use crate::{VoxjSerdePositionBlock, VoxjSerdeSampleBlock};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One voxel volume: pure geometry, placed only by a hierarchy node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjSerdeObject {
    /// Display name of the object.
    pub name: String,

    /// Indices into [`VoxjMain::palettes`](crate::VoxjMain::palettes), in
    /// resolution order.
    pub palette_refs: Vec<usize>,

    /// `[X, Y, Z]` size in voxels; every voxel lies in
    /// `[0, X) x [0, Y) x [0, Z)`.
    pub bounds: [u32; 3],

    /// Encoded voxel positions; the chosen encoding fixes the object's canonical
    /// voxel order.
    pub voxel_positions: VoxjSerdePositionBlock,

    /// Encoded voxel samples, one cell index per referenced palette, in the
    /// position block's voxel order.
    pub voxel_samples: VoxjSerdeSampleBlock,
}
