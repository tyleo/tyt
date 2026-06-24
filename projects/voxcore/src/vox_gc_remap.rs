use crate::{BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell};
use branded_id::{U32Id, soa::IdRemap};
use std::collections::HashMap;

/// The id relabelings from a [`VoxState::gc`](crate::VoxState::gc), one per pool,
/// for translating ids held across the call.
///
/// `cells` is keyed by each palette's old id, since a cell id is only meaningful
/// within its palette.
pub struct VoxGcRemap {
    /// The object pool relabeling.
    pub objects: IdRemap<BVoxObject, u32>,

    /// The palette pool relabeling.
    pub palettes: IdRemap<BVoxPalette, u32>,

    /// The hierarchy node pool relabeling.
    pub hierarchy_nodes: IdRemap<BVoxHierarchyNode, u32>,

    /// Each palette's cell relabeling, keyed by the palette's pre-gc id.
    pub cells: HashMap<U32Id<BVoxPalette>, IdRemap<BVoxPaletteCell, u32>>,
}
