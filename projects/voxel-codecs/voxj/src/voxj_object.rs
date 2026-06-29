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

    /// Indices into
    /// [`VoxjRuntimeState::palettes`](crate::VoxjRuntimeState::palettes), in
    /// resolution order.
    pub palette_refs: Vec<usize>,

    /// `[X, Y, Z]` size in voxels, exactly tight around the geometry: every
    /// voxel lies in `[0, X) x [0, Y) x [0, Z)` and some voxel reaches each end
    /// of every axis, so the grid carries no empty margin. An empty object is
    /// `[0, 0, 0]`; build-volume margin lives in the edit grid
    /// ([`VoxjEditObject`](crate::VoxjEditObject)).
    pub bounds: [u32; 3],

    /// `[X, Y, Z]` translation in voxels from the placing hierarchy node to the
    /// grid's min corner. `[0, 0, 0]` puts the min corner at the node origin.
    #[cfg_attr(feature = "serde", serde(default))]
    pub origin: [i32; 3],

    /// Encoded voxel positions; the encoding fixes the voxel order.
    pub voxel_positions: VoxjPositionBlock,

    /// Encoded voxel samples, one cell index per referenced palette, in voxel
    /// order.
    pub voxel_samples: VoxjSampleBlock,
}
