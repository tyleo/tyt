use branded_id::U32Id;
use std::{any::Any, collections::HashMap, fmt::Debug};
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

    fn object_did_retain(&mut self, main: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        (**self).object_did_retain(main, object_id)
    }

    fn object_will_release(&mut self, main: &VoxState, object_id: U32Id<BVoxObject>) -> Result<()> {
        (**self).object_will_release(main, object_id)
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
