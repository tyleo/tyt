use crate::ext::MVoxExt;
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{Result, VoxExt, encode_entry},
};

/// The MagicaVoxel ext as a state's ext. Its block is the `mvox` entry.
impl VoxExt for MVoxExt {
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
