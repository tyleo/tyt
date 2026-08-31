use crate::{Result, VoxelMaxExt, VoxjExtCodec, keyed_ext_from_voxj, keyed_voxj_ext};
use voxcore::VoxMap;

/// Keeps the Voxel Max ext under the `voxel-max` key of a document's `ext`
/// block.
impl VoxjExtCodec for VoxelMaxExt {
    fn to_voxj_ext(&self) -> Result<VoxMap> {
        keyed_voxj_ext("voxel-max", self)
    }

    fn from_voxj_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_voxj("voxel-max", ext)
    }
}
