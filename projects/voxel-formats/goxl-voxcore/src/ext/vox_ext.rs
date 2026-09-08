use crate::ext::GoxlExt;
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{Result, VoxExt, encode_entry},
};

/// The Goxel ext as a state's ext. Its block is the `goxl` entry.
impl VoxExt for GoxlExt {
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
