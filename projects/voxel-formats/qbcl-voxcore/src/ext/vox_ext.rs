use crate::ext::{QbExt, QbclExt, QbtExt};
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{FollowListing, Result, VoxExt, encode_entry},
};

/// The Qubicle Binary ext as a state's ext. Its block is the `qb` entry. The
/// matrices follow the object listing. A retained object takes no entry. The
/// writer fills it in like a synthesized matrix. The visibility bytes do not
/// follow the voxel hooks. A live voxel retained again for a repaint fires
/// the same hook as a new one, so the list cannot tell an insert from a
/// repaint. The writer's count check reports a list out of step.
impl VoxExt for QbExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }

    fn object_did_retain(&mut self, index: usize) {
        self.matrices.follow_retain(index);
    }

    fn object_will_release(&mut self, index: usize) {
        self.matrices.follow_release(index);
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        self.matrices.follow_move(from, to);
    }
}

/// The Qubicle Binary Tree ext as a state's ext. Its block is the `qbt` entry.
impl VoxExt for QbtExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }
}

/// The Qubicle Project ext as a state's ext. Its block is the `qbcl` entry.
impl VoxExt for QbclExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }
}
