use crate::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxObject, VoxPalette, VoxValue,
};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};

/// The in-memory state of a voxel model: its objects, shared palettes, scene
/// hierarchy, and roots.
///
/// Objects, palettes, and hierarchy nodes are generated from their id pools and
/// held in columns that track liveness through those pools, so this type does
/// not derive `Clone` or `PartialEq`; its [`Drop`] releases the columns.
#[derive(Debug, Default)]
pub struct VoxState {
    /// The object id pool.
    pub object_ids: IdStruct<BVoxObject>,

    /// The objects, keyed by `U32Id<BVoxObject>`.
    pub objects: IdField<BVoxObject, VoxObject>,

    /// The palette id pool.
    pub palette_ids: IdStruct<BVoxPalette>,

    /// The shared palettes, keyed by `U32Id<BVoxPalette>`.
    pub palettes: IdField<BVoxPalette, VoxPalette>,

    /// The hierarchy node id pool.
    pub hierarchy_node_ids: IdStruct<BVoxHierarchyNode>,

    /// The hierarchy nodes, keyed by `U32Id<BVoxHierarchyNode>`.
    pub hierarchy_nodes: IdField<BVoxHierarchyNode, VoxHierarchyNode>,

    /// The scene's roots: ids into [`hierarchy_nodes`](Self::hierarchy_nodes).
    pub root_hierarchy_nodes: Vec<U32Id<BVoxHierarchyNode>>,

    /// An optional namespace for user-defined extensions. The core format
    /// assigns it no meaning.
    pub ext: Option<VoxValue>,
}

impl Drop for VoxState {
    fn drop(&mut self) {
        // Safety: each column retains a value for every id in its paired pool.
        // The fields free their own storage on drop, so releasing the values is
        // all that is needed here.
        unsafe {
            self.objects.release_all(&self.object_ids);
            self.palettes.release_all(&self.palette_ids);
            self.hierarchy_nodes.release_all(&self.hierarchy_node_ids);
        }
    }
}
