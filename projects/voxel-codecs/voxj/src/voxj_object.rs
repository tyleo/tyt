use crate::{VoxjPositionBlock, VoxjSampleBlock};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One object's voxel geometry and per-palette samples, in encoded blocks.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjObject {
    /// Display name of the object.
    pub name: String,

    /// Indices into [`VoxjMain::palettes`](crate::VoxjMain::palettes), in
    /// resolution order.
    pub palette_refs: Vec<usize>,

    /// `[X, Y, Z]` size in voxels; every voxel lies in
    /// `[0, X) x [0, Y) x [0, Z)`.
    pub bounds: [u32; 3],

    /// Encoded voxel positions; the encoding fixes the voxel order.
    pub voxel_positions: VoxjPositionBlock,

    /// Encoded voxel samples, one cell index per referenced palette, in voxel
    /// order.
    pub voxel_samples: VoxjSampleBlock,
}
