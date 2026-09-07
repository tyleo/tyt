use crate::MVoxExt;
use voxcore::{
    VoxValue,
    ext::{
        Result, VoxExtEntryCodec,
        json::{ext_from_vox_value, vox_value_from_ext},
    },
};

/// Keeps the MagicaVoxel ext under the `mvox` key of a document's `ext` block.
impl VoxExtEntryCodec for MVoxExt {
    const KEY: &'static str = "mvox";

    fn to_vox_ext_entry(&self) -> Result<VoxValue> {
        vox_value_from_ext(self)
    }

    fn from_vox_ext_entry(value: &VoxValue) -> Result<Self> {
        ext_from_vox_value(value)
    }
}
