use crate::{BVoxObject, VoxEditObject};
use branded_id::{
    U32Id,
    soa::{IdField, IdRemap},
};

/// The editor state of a voxel model: one [`VoxEditObject`] per runtime object,
/// keyed by object id and aligned with the objects in
/// [`VoxRuntimeState`](crate::VoxRuntimeState). Held by
/// [`VoxState`](crate::VoxState), which keeps it in lockstep with the object
/// pool; read and write the grids through
/// [`VoxState::edit_object`](crate::VoxState::edit_object) and
/// [`VoxState::set_edit_object`](crate::VoxState::set_edit_object).
///
/// `VoxEditObject` is `Copy`, so the backing column needs no destructor and the
/// whole state clones with a derived `Clone`.
#[derive(Clone, Debug, Default)]
pub struct VoxEditState {
    /// Per object-id edit grid, paired with the object pool in the runtime
    /// state. A retained object id always has an edit grid here.
    edit_objects: IdField<BVoxObject, VoxEditObject>,
}

impl VoxEditState {
    /// Stores the edit grid for a freshly retained object id.
    pub(crate) fn retain(&mut self, id: U32Id<BVoxObject>, edit_object: VoxEditObject) {
        self.edit_objects.retain(id, edit_object);
    }

    /// The edit grid for `id`.
    ///
    /// # Safety
    /// `id` must be retained in the paired object pool.
    pub(crate) unsafe fn get(&self, id: U32Id<BVoxObject>) -> VoxEditObject {
        *unsafe { self.edit_objects.get(id) }
    }

    /// Overwrites the edit grid for `id`.
    ///
    /// # Safety
    /// `id` must be retained in the paired object pool.
    pub(crate) unsafe fn set(&mut self, id: U32Id<BVoxObject>, edit_object: VoxEditObject) {
        unsafe { self.edit_objects.set(id, edit_object) };
    }

    /// Releases the edit grid for `id`, paired with releasing the object.
    ///
    /// # Safety
    /// `id` must be retained in the paired object pool.
    pub(crate) unsafe fn release(&mut self, id: U32Id<BVoxObject>) {
        unsafe { self.edit_objects.release(id) };
    }

    /// Compacts the column to a just-`gc`'d object pool.
    ///
    /// # Safety
    /// `remap` must be the value returned by the object pool's `gc`, with this
    /// column still in sync with the pre-gc state.
    pub(crate) unsafe fn gc(&mut self, remap: &IdRemap<BVoxObject, u32>) {
        unsafe { self.edit_objects.gc(remap) };
    }
}
