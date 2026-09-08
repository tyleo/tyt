use crate::ext::{QbExt, QbclExt, QbtExt};
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{Result, VoxExt, encode_entry},
};

/// The Qubicle Binary ext as a state's ext. Its block is the `qb` entry.
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
