use crate::{QbExt, QbclExt, QbtExt};
use voxcore::{
    VoxValue,
    ext::{
        Result, VoxExtEntryCodec,
        json::{ext_from_vox_value, vox_value_from_ext},
    },
};

/// Keeps the Qubicle Binary ext under the `qb` key of a document's `ext` block.
impl VoxExtEntryCodec for QbExt {
    const KEY: &'static str = "qb";

    fn to_vox_ext_entry(&self) -> Result<VoxValue> {
        vox_value_from_ext(self)
    }

    fn from_vox_ext_entry(value: &VoxValue) -> Result<Self> {
        ext_from_vox_value(value)
    }
}

/// Keeps the Qubicle Binary Tree ext under the `qbt` key of a document's `ext`
/// block.
impl VoxExtEntryCodec for QbtExt {
    const KEY: &'static str = "qbt";

    fn to_vox_ext_entry(&self) -> Result<VoxValue> {
        vox_value_from_ext(self)
    }

    fn from_vox_ext_entry(value: &VoxValue) -> Result<Self> {
        ext_from_vox_value(value)
    }
}

/// Keeps the Qubicle Project ext under the `qbcl` key of a document's `ext`
/// block.
impl VoxExtEntryCodec for QbclExt {
    const KEY: &'static str = "qbcl";

    fn to_vox_ext_entry(&self) -> Result<VoxValue> {
        vox_value_from_ext(self)
    }

    fn from_vox_ext_entry(value: &VoxValue) -> Result<Self> {
        ext_from_vox_value(value)
    }
}
