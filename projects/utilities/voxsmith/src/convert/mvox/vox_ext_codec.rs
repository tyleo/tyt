use crate::MagicaVoxelExt;
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the MagicaVoxel ext under the `magica-voxel` key of a document's
/// `ext` block.
impl VoxExtCodec for MagicaVoxelExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("magica-voxel", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("magica-voxel", ext)
    }
}
