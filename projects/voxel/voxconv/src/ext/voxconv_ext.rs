use branded_id::U32Id;
use std::{any::Any, collections::HashMap, fmt::Debug};
use ty_math::{TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxVoxel, Result, VoxExt,
    VoxGcRemap, VoxState,
};

/// The ext a [`VoxconvVoxMain`](crate::ext::VoxconvVoxMain) boxes: a
/// [`VoxExt`] that can also downcast and clone, so the box can be opened to
/// the format's ext and a boxed state cloned. Every `VoxExt` that is `Any`,
/// `Debug`, and `Clone` implements it through the blanket impl. The box
/// forwards every hook to the ext it holds.
pub trait VoxconvExt: VoxExt + Any + Debug {
    /// Clones the ext into a box.
    fn clone_box(&self) -> Box<dyn VoxconvExt>;
}

impl<E: VoxExt + Any + Debug + Clone> VoxconvExt for E {
    fn clone_box(&self) -> Box<dyn VoxconvExt> {
        Box::new(self.clone())
    }
}

impl dyn VoxconvExt {
    /// Whether the ext is an `E`.
    pub fn is<E: Any>(&self) -> bool {
        (self as &dyn Any).is::<E>()
    }

    /// The ext as an `E`, or `None` when it is another type.
    pub fn downcast_ref<E: Any>(&self) -> Option<&E> {
        (self as &dyn Any).downcast_ref::<E>()
    }
}

impl Clone for Box<dyn VoxconvExt> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl VoxExt for Box<dyn VoxconvExt> {
    fn hierarchy_node_did_retain(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        (**self).hierarchy_node_did_retain(main, node_id)
    }

    fn hierarchy_node_will_release(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        (**self).hierarchy_node_will_release(main, node_id)
    }

    fn hierarchy_node_name_did_set(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_name: &str,
    ) -> Result<()> {
        (**self).hierarchy_node_name_did_set(main, node_id, old_name)
    }

    fn hierarchy_node_transform_did_set(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_transform: TyTransformF64,
    ) -> Result<()> {
        (**self).hierarchy_node_transform_did_set(main, node_id, old_transform)
    }

    fn hierarchy_node_children_did_set(
        &mut self,
        main: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_child_node_ids: &[U32Id<BVoxHierarchyNode>],
        old_child_object_ids: &[U32Id<BVoxObject>],
    ) -> Result<()> {
        (**self).hierarchy_node_children_did_set(
            main,
            node_id,
            old_child_node_ids,
            old_child_object_ids,
        )
    }

    fn root_hierarchy_node_ids_did_set(
        &mut self,
        main: &VoxState,
        old_root_ids: &[U32Id<BVoxHierarchyNode>],
    ) -> Result<()> {
        (**self).root_hierarchy_node_ids_did_set(main, old_root_ids)
    }

    fn object_did_retain(&mut self, main: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        (**self).object_did_retain(main, object_id)
    }

    fn object_will_release(&mut self, main: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        (**self).object_will_release(main, object_id)
    }

    fn object_did_move(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_index: usize,
    ) -> Result<()> {
        (**self).object_did_move(main, object_id, old_index)
    }

    fn object_name_did_set(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_name: &str,
    ) -> Result<()> {
        (**self).object_name_did_set(main, object_id, old_name)
    }

    fn object_origin_did_set(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_origin: TyVector3I32,
    ) -> Result<()> {
        (**self).object_origin_did_set(main, object_id, old_origin)
    }

    fn object_voxels_did_remap(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        voxel_ids: &HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>,
    ) -> Result<()> {
        (**self).object_voxels_did_remap(main, object_id, old_bounds, voxel_ids)
    }

    fn object_voxels_did_resample(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        old_voxel_ids: &[U32Id<BVoxVoxel>],
    ) -> Result<()> {
        (**self).object_voxels_did_resample(main, object_id, old_bounds, old_voxel_ids)
    }

    fn palette_did_retain(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        (**self).palette_did_retain(main, palette_id)
    }

    fn palette_will_release(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        (**self).palette_will_release(main, palette_id)
    }

    fn material_did_retain(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
    ) -> Result<()> {
        (**self).material_did_retain(main, palette_id, material_id)
    }

    fn materials_will_release(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        (**self).materials_will_release(main, palette_id, material_ids)
    }

    fn materials_did_repaint(
        &mut self,
        main: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
    ) -> Result<()> {
        (**self).materials_did_repaint(main, palette_id, replacement_ids)
    }

    fn voxel_did_retain(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        (**self).voxel_did_retain(main, object_id, voxel_id)
    }

    fn voxel_will_release(
        &mut self,
        main: &VoxState,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        (**self).voxel_will_release(main, object_id, voxel_id)
    }

    fn did_gc(&mut self, main: &VoxState, remap: &VoxGcRemap) -> Result<()> {
        (**self).did_gc(main, remap)
    }
}
