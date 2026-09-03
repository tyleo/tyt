use crate::VoxelMaxExt;
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the Voxel Max ext under the `voxel-max` key of a document's `ext`
/// block.
impl VoxExtCodec for VoxelMaxExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("voxel-max", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("voxel-max", ext)
    }
}
