use crate::{
    Result,
    ext::{VoxconvExt, ext_from_vox_value},
};
use serde::de::DeserializeOwned;
use voxcore::VoxMapEntry;

/// The `E` a slot decodes to when its key is `key`, boxed, or `None` when
/// the slot sits under another key. A slot under `key` that does not decode
/// as `E` is an error.
pub fn ext_from_slot<E: VoxconvExt + DeserializeOwned>(
    key: &str,
    slot: &VoxMapEntry,
) -> Result<Option<Box<dyn VoxconvExt>>> {
    if slot.key != key {
        return Ok(None);
    }

    let ext: Box<dyn VoxconvExt> = Box::new(ext_from_vox_value::<E>(&slot.value)?);

    Ok(Some(ext))
}
