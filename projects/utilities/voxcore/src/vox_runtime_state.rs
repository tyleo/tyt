use crate::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxValuePool, VoxHierarchyNode, VoxObject,
    VoxPalette, VoxValuePool,
};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};

/// The runtime scene of a voxel model, held by [`VoxMain`](crate::VoxMain)
/// alongside the ext.
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

impl VoxRuntimeState {
    /// Deep copy, rebuilding every column against fresh id pools because the
    /// SoA types can't derive `Clone`.
    pub(crate) fn clone_runtime_state(&self) -> Self {
        let mut value_pools = IdField::new();
        for value_pool_id in self.value_pool_ids.iter() {
            // Safety: retained ids have a value.
            value_pools.retain(
                value_pool_id,
                unsafe { self.value_pools.get(value_pool_id) }.clone_value_pool(),
            );
        }

        let mut palettes = IdField::new();
        for palette_id in self.palette_ids.iter() {
            // Safety: retained ids have a value.
            palettes.retain(
                palette_id,
                unsafe { self.palettes.get(palette_id) }.clone_palette(),
            );
        }

        let mut objects = IdField::new();
        for object_id in self.object_ids.iter() {
            // Safety: retained ids have a value.
            objects.retain(
                object_id,
                unsafe { self.objects.get(object_id) }.clone_object(),
            );
        }

        let mut hierarchy_nodes = IdField::new();
        for node_id in self.hierarchy_node_ids.iter() {
            // Safety: retained ids have a value.
            hierarchy_nodes.retain(
                node_id,
                unsafe { self.hierarchy_nodes.get(node_id) }.clone(),
            );
        }

        Self {
            value_pool_ids: self.value_pool_ids.clone(),
            value_pools,
            palette_ids: self.palette_ids.clone(),
            palettes,
            hierarchy_node_ids: self.hierarchy_node_ids.clone(),
            hierarchy_nodes,
            object_ids: self.object_ids.clone(),
            objects,
            root_hierarchy_node_ids: self.root_hierarchy_node_ids.clone(),
        }
    }
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
