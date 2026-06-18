use crate::{PositionBlock, SampleBlock};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One voxel volume: pure geometry, placed only by a hierarchy node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjObject {
    pub name: String,
    /// Indices into `VoxjMain::palettes`, in resolution order.
    pub palette_refs: Vec<usize>,
    /// `[X, Y, Z]` size in voxels; every voxel lies in `[0, X) x [0, Y) x [0, Z)`.
    pub bounds: [u32; 3],
    pub voxel_positions: PositionBlock,
    pub voxel_samples: SampleBlock,
}
