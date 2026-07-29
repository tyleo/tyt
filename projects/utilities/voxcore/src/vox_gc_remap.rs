use crate::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePool, BVoxValuePoolValue,
};
use branded_id::{IdVec, soa::IdRemap};

/// The id relabelings from a [`VoxMain::gc`](crate::VoxMain::gc), one per pool,
/// for translating ids held across the call.
pub struct VoxGcRemap {
    /// The value-pool relabeling.
    pub value_pools: IdRemap<BVoxValuePool, u32>,

    /// Each value pool's value relabeling, indexed by the pool's old id.
    pub value_pool_values: IdVec<BVoxValuePool, IdRemap<BVoxValuePoolValue, u32>>,

    /// The palette pool relabeling.
    pub palettes: IdRemap<BVoxPalette, u32>,

    /// Each palette's material relabeling, indexed by the palette's old id,
    /// empty where a palette was removed.
    pub materials: IdVec<BVoxPalette, IdRemap<BVoxMaterial, u32>>,

    /// The object pool relabeling.
    pub objects: IdRemap<BVoxObject, u32>,

    /// The hierarchy node pool relabeling.
    pub hierarchy_nodes: IdRemap<BVoxHierarchyNode, u32>,
}
