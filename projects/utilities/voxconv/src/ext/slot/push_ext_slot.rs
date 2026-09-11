use crate::{
    Result,
    ext::{VoxconvExt, push_slot, vox_value_from_ext},
};
use serde::Serialize;
use voxcore::VoxMapEntry;

/// Appends the slot under `key` a boxed ext encodes to when it holds an
/// `E`, and reports whether it did. Errors when the block already holds the
/// key.
pub fn push_ext_slot<E: VoxconvExt + Serialize>(
    key: &str,
    ext: &dyn VoxconvExt,
    slots: &mut Vec<VoxMapEntry>,
) -> Result<bool> {
    let Some(ext) = ext.downcast_ref::<E>() else {
        return Ok(false);
    };

    push_slot(
        slots,
        VoxMapEntry {
            key: key.to_owned(),
            value: vox_value_from_ext(ext)?,
        },
    )?;

    Ok(true)
}
