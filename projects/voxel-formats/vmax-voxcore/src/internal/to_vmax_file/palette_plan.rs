use crate::VoxelIndices;
use branded_id::U32Id;
use std::collections::BTreeMap;
use vmax::VMaxMaterial;
use voxcore::BVoxMaterial;

/// The Voxel Max palette one voxcore palette writes, shared by every object
/// layering it.
pub(crate) struct PalettePlan {
    /// The `pal` filename the objects reference.
    pub(crate) pal: String,

    /// The display name for the settings sidecar.
    pub(crate) name: String,

    /// The color table, the `baseColor` value pool whole. `None` for a
    /// colorless palette, which borrows a name and writes no file.
    pub(crate) color_table: Option<Vec<[u8; 4]>>,

    /// The indices each material writes.
    pub(crate) indices: BTreeMap<U32Id<BVoxMaterial>, VoxelIndices>,

    /// The materials in slot order. Empty when the palette binds no material
    /// axis, which leaves Voxel Max its own defaults.
    pub(crate) materials: Vec<VMaxMaterial>,
}
