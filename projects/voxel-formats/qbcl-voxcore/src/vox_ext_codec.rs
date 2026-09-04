use crate::{QbExt, QbclExt, QbtExt};
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the Qubicle Binary ext under the `qb` key of a document's
/// `ext` block.
impl VoxExtCodec for QbExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("qb", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("qb", ext)
    }
}

/// Keeps the Qubicle Binary Tree ext under the `qbt` key of a
/// document's `ext` block.
impl VoxExtCodec for QbtExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("qbt", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("qbt", ext)
    }
}

/// Keeps the Qubicle Project ext under the `qbcl` key of a document's
/// `ext` block.
impl VoxExtCodec for QbclExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("qbcl", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("qbcl", ext)
    }
}
