use crate::VMaxExt;
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the Voxel Max ext under the `vmax` key of a document's `ext`
/// block.
impl VoxExtCodec for VMaxExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("vmax", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("vmax", ext)
    }
}
