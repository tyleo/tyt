use crate::{MagicaVoxelExt, Result, VoxjExtCodec, keyed_ext_from_voxj, keyed_voxj_ext};
use voxj::VoxjMap;

/// Keeps the MagicaVoxel ext under the `magica-voxel` key of a document's
/// `ext` block.
impl VoxjExtCodec for MagicaVoxelExt {
    fn to_voxj_ext(&self) -> Result<VoxjMap> {
        keyed_voxj_ext("magica-voxel", self)
    }

    fn from_voxj_ext(ext: &VoxjMap) -> Result<Option<Self>> {
        keyed_ext_from_voxj("magica-voxel", ext)
    }
}
