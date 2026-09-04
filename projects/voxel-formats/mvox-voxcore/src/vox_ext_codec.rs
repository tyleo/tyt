use crate::MVoxExt;
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the MagicaVoxel ext under the `mvox` key of a document's
/// `ext` block.
impl VoxExtCodec for MVoxExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("mvox", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("mvox", ext)
    }
}
