use crate::{MaterialSlotKey, VoxelIndices};
use branded_id::U32Id;
use std::collections::{BTreeMap, HashMap};
use vmax::VMaxMaterial;
use voxcore::BVoxMaterial;

/// The Voxel Max palette one ordered list of layer palettes writes. Objects
/// layering the same palettes share it, so it grows as each is visited and
/// writes once every object has.
pub(crate) struct PalettePlan {
    /// The `pal` filename the objects reference.
    pub(crate) pal: String,

    /// The display name for the settings sidecar.
    pub(crate) name: String,

    /// The color table, the `baseColor` supplier's value pool whole. `None`
    /// for a colorless palette, which borrows a name and writes no file.
    pub(crate) color_table: Option<Vec<[u8; 4]>>,

    /// The slot each material draws, for a one-layer palette loaded from
    /// Voxel Max with its exact material list. `None` derives the slots.
    pub(crate) exact_slots: Option<BTreeMap<U32Id<BVoxMaterial>, u8>>,

    /// The indices of every sample tuple seen so far.
    pub(crate) samples: HashMap<Vec<U32Id<BVoxMaterial>>, VoxelIndices>,

    /// The derived slot of each key seen so far. Empty under exact slots.
    pub(crate) slot_index_of: HashMap<MaterialSlotKey, u8>,

    /// The materials in slot order. Empty when no layer supplies a
    /// material-axis property, which leaves Voxel Max its own defaults.
    pub(crate) materials: Vec<VMaxMaterial>,
}
