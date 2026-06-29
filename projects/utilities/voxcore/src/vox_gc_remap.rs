use crate::{BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell};
use branded_id::{IdVec, soa::IdRemap};

/// The id relabelings from a [`VoxMain::gc`](crate::VoxMain::gc), one per pool,
/// for translating ids held across the call.
///
/// `cells` is indexed by each palette's old id, since a cell id is only
/// meaningful within its palette.
pub struct VoxGcRemap {
    /// The object pool relabeling.
    pub objects: IdRemap<BVoxObject, u32>,

    /// The palette pool relabeling.
    pub palettes: IdRemap<BVoxPalette, u32>,

    /// The hierarchy node pool relabeling.
    pub hierarchy_nodes: IdRemap<BVoxHierarchyNode, u32>,

    /// Each palette's cell relabeling, indexed by the palette's old id, empty
    /// where a palette was removed.
    pub cells: IdVec<BVoxPalette, IdRemap<BVoxPaletteCell, u32>>,
}
