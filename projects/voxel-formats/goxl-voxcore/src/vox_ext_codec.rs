use crate::GoxlExt;
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the Goxel ext under the `goxl` key of a document's `ext` block.
impl VoxExtCodec for GoxlExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("goxl", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("goxl", ext)
    }
}
