use crate::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxValuePool, VoxHierarchyNode, VoxObject,
    VoxPalette, VoxValuePool,
};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};

/// The runtime scene of a voxel model, held by [`VoxMain`](crate::VoxMain).
///
/// This is the struct-of-arrays backing store. [`VoxMain`](crate::VoxMain) owns
/// mutation logic over these fields; they are crate-private so the id pools and
/// columns stay in sync.
#[derive(Debug, Default)]
pub struct VoxRuntimeState {
    /// Value-pool id pool.
    pub(crate) value_pool_ids: IdStruct<BVoxValuePool>,

    /// The shared value pools.
    pub(crate) value_pools: IdField<BVoxValuePool, VoxValuePool>,

    /// Palette id pool.
    pub(crate) palette_ids: IdStruct<BVoxPalette>,

    /// The shared palettes.
    pub(crate) palettes: IdField<BVoxPalette, VoxPalette>,

    /// Object id pool.
    pub(crate) object_ids: IdStruct<BVoxObject>,

    /// The objects.
    pub(crate) objects: IdField<BVoxObject, VoxObject>,

    /// Hierarchy node id pool.
    pub(crate) hierarchy_node_ids: IdStruct<BVoxHierarchyNode>,

    /// The hierarchy nodes.
    pub(crate) hierarchy_nodes: IdField<BVoxHierarchyNode, VoxHierarchyNode>,

    /// The scene's roots: hierarchy node ids.
    pub(crate) root_hierarchy_node_ids: Vec<U32Id<BVoxHierarchyNode>>,
}

impl Drop for VoxRuntimeState {
    fn drop(&mut self) {
        // Safety: each column holds a value for every id in its id pool; the
        // fields free their own storage on drop.
        unsafe {
            self.value_pools.release_all(&self.value_pool_ids);
            self.palettes.release_all(&self.palette_ids);
            self.objects.release_all(&self.object_ids);
            self.hierarchy_nodes.release_all(&self.hierarchy_node_ids);
        }
    }
}
