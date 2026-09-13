use branded_id::U32Id;
use meshdoc::{
    BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive, BMeshTexture,
    MeshExt, MeshGcRemap, MeshState, Result,
};
use std::{any::Any, fmt::Debug};

/// The ext a [`MeshconvMeshMain`](crate::ext::MeshconvMeshMain) boxes: a
/// [`MeshExt`] that can also downcast and clone, so the box can be opened
/// to the format's ext and a boxed state cloned. Every `MeshExt` that is
/// `Any`, `Debug`, and `Clone` implements it through the blanket impl. The
/// box forwards every hook to the ext it holds.
pub trait MeshconvExt: MeshExt + Any + Debug {
    /// Clones the ext into a box.
    fn clone_box(&self) -> Box<dyn MeshconvExt>;
}

impl<E: MeshExt + Any + Debug + Clone> MeshconvExt for E {
    fn clone_box(&self) -> Box<dyn MeshconvExt> {
        Box::new(self.clone())
    }
}

impl dyn MeshconvExt {
    /// Whether the ext is an `E`.
    pub fn is<E: Any>(&self) -> bool {
        (self as &dyn Any).is::<E>()
    }

    /// The ext as an `E`, or `None` when it is another type.
    pub fn downcast_ref<E: Any>(&self) -> Option<&E> {
        (self as &dyn Any).downcast_ref::<E>()
    }
}

impl Clone for Box<dyn MeshconvExt> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl MeshExt for Box<dyn MeshconvExt> {
    fn hierarchy_node_did_retain(
        &mut self,
        state: &MeshState,
        node_id: U32Id<BMeshHierarchyNode>,
    ) -> Result<()> {
        (**self).hierarchy_node_did_retain(state, node_id)
    }

    fn hierarchy_node_will_release(
        &mut self,
        state: &MeshState,
        node_id: U32Id<BMeshHierarchyNode>,
    ) -> Result<()> {
        (**self).hierarchy_node_will_release(state, node_id)
    }

    fn object_did_retain(
        &mut self,
        state: &MeshState,
        object_id: U32Id<BMeshObject>,
    ) -> Result<()> {
        (**self).object_did_retain(state, object_id)
    }

    fn object_will_release(
        &mut self,
        state: &MeshState,
        object_id: U32Id<BMeshObject>,
    ) -> Result<()> {
        (**self).object_will_release(state, object_id)
    }

    fn primitive_did_retain(
        &mut self,
        state: &MeshState,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
    ) -> Result<()> {
        (**self).primitive_did_retain(state, object_id, primitive_id)
    }

    fn primitive_will_release(
        &mut self,
        state: &MeshState,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
    ) -> Result<()> {
        (**self).primitive_will_release(state, object_id, primitive_id)
    }

    fn material_did_retain(
        &mut self,
        state: &MeshState,
        material_id: U32Id<BMeshMaterial>,
    ) -> Result<()> {
        (**self).material_did_retain(state, material_id)
    }

    fn material_will_release(
        &mut self,
        state: &MeshState,
        material_id: U32Id<BMeshMaterial>,
    ) -> Result<()> {
        (**self).material_will_release(state, material_id)
    }

    fn texture_did_retain(
        &mut self,
        state: &MeshState,
        texture_id: U32Id<BMeshTexture>,
    ) -> Result<()> {
        (**self).texture_did_retain(state, texture_id)
    }

    fn texture_will_release(
        &mut self,
        state: &MeshState,
        texture_id: U32Id<BMeshTexture>,
    ) -> Result<()> {
        (**self).texture_will_release(state, texture_id)
    }

    fn image_did_retain(&mut self, state: &MeshState, image_id: U32Id<BMeshImage>) -> Result<()> {
        (**self).image_did_retain(state, image_id)
    }

    fn image_will_release(&mut self, state: &MeshState, image_id: U32Id<BMeshImage>) -> Result<()> {
        (**self).image_will_release(state, image_id)
    }

    fn did_gc(&mut self, state: &MeshState, remap: &MeshGcRemap) -> Result<()> {
        (**self).did_gc(state, remap)
    }
}
