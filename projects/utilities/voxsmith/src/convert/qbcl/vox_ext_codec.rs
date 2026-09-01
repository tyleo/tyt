use crate::{QubicleQbExt, QubicleQbclExt, QubicleQbtExt};
use voxcore::{
    VoxMap,
    ext::{
        Result, VoxExtCodec,
        json::{keyed_ext_from_vox, keyed_vox_ext},
    },
};

/// Keeps the Qubicle Binary ext under the `qubicle-qb` key of a document's
/// `ext` block.
impl VoxExtCodec for QubicleQbExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("qubicle-qb", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("qubicle-qb", ext)
    }
}

/// Keeps the Qubicle Binary Tree ext under the `qubicle-qbt` key of a
/// document's `ext` block.
impl VoxExtCodec for QubicleQbtExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("qubicle-qbt", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("qubicle-qbt", ext)
    }
}

/// Keeps the Qubicle Project ext under the `qubicle-qbcl` key of a document's
/// `ext` block.
impl VoxExtCodec for QubicleQbclExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        keyed_vox_ext("qubicle-qbcl", self)
    }

    fn from_vox_ext(ext: &VoxMap) -> Result<Option<Self>> {
        keyed_ext_from_vox("qubicle-qbcl", ext)
    }
}
