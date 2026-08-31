use crate::{GoxelExt, Result, VoxjExtCodec, keyed_ext_from_voxj, keyed_voxj_ext};
use voxcore::VoxMap;

/// Keeps the Goxel ext under the `goxel` key of a document's `ext` block.
impl VoxjExtCodec for GoxelExt {
    fn to_voxj_ext(&self) -> Result<VoxMap> {
        keyed_voxj_ext("goxel", self)
    }

    fn from_voxj_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_voxj("goxel", ext)
    }
}
