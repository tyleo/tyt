use branded_id::U32Id;
use std::{any::Any, fmt::Debug};
use voxcore::{BVoxVoxel, VoxExt};

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
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        (**self).hierarchy_node_did_retain(index);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        (**self).hierarchy_node_will_release(index);
    }

    fn object_did_retain(&mut self, index: usize) {
        (**self).object_did_retain(index);
    }

    fn object_will_release(&mut self, index: usize) {
        (**self).object_will_release(index);
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        (**self).object_did_move(from, to);
    }

    fn palette_did_retain(&mut self, index: usize) {
        (**self).palette_did_retain(index);
    }

    fn palette_will_release(&mut self, index: usize) {
        (**self).palette_will_release(index);
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        (**self).palette_did_move(from, to);
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        (**self).material_did_retain(palette, index);
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        (**self).materials_will_release(palette, indices);
    }

    fn materials_did_repaint(&mut self, palette: usize, remap: &[(usize, usize)]) {
        (**self).materials_did_repaint(palette, remap);
    }

    fn voxel_did_retain(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        (**self).voxel_did_retain(object, voxel);
    }

    fn voxel_will_release(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        (**self).voxel_will_release(object, voxel);
    }
}
