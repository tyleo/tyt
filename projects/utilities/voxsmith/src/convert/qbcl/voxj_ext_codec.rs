use crate::{
    QubicleQbExt, QubicleQbclExt, QubicleQbtExt, Result, VoxjExtCodec, keyed_ext_from_voxj,
    keyed_voxj_ext,
};
use voxj::VoxjMap;

/// Keeps the Qubicle Binary ext under the `qubicle-qb` key of a document's
/// `ext` block.
impl VoxjExtCodec for QubicleQbExt {
    fn to_voxj_ext(&self) -> Result<VoxjMap> {
        keyed_voxj_ext("qubicle-qb", self)
    }

    fn from_voxj_ext(ext: &VoxjMap) -> Result<Option<Self>> {
        keyed_ext_from_voxj("qubicle-qb", ext)
    }
}

/// Keeps the Qubicle Binary Tree ext under the `qubicle-qbt` key of a
/// document's `ext` block.
impl VoxjExtCodec for QubicleQbtExt {
    fn to_voxj_ext(&self) -> Result<VoxjMap> {
        keyed_voxj_ext("qubicle-qbt", self)
    }

    fn from_voxj_ext(ext: &VoxjMap) -> Result<Option<Self>> {
        keyed_ext_from_voxj("qubicle-qbt", ext)
    }
}

/// Keeps the Qubicle Project ext under the `qubicle-qbcl` key of a document's
/// `ext` block.
impl VoxjExtCodec for QubicleQbclExt {
    fn to_voxj_ext(&self) -> Result<VoxjMap> {
        keyed_voxj_ext("qubicle-qbcl", self)
    }

    fn from_voxj_ext(ext: &VoxjMap) -> Result<Option<Self>> {
        keyed_ext_from_voxj("qubicle-qbcl", ext)
    }
}
