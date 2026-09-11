use crate::{
    Result,
    ext::{VoxconvExt, ext_from_vox_value, push_slots},
};
use serde::de::DeserializeOwned;

/// The format ext `E` in a boxed ext, cloned out. A box holding an `E` gives
/// it as it is. Any other box encodes to its Voxel Json `ext` block slots,
/// and the slot under `key` decodes. A box with no such slot, such as
/// another format's ext, gives `None`.
pub fn find_ext<E: VoxconvExt + Clone + DeserializeOwned>(
    key: &str,
    ext: &dyn VoxconvExt,
) -> Result<Option<E>> {
    if let Some(ext) = ext.downcast_ref::<E>() {
        return Ok(Some(ext.clone()));
    }

    let mut slots = Vec::new();

    push_slots(ext, &mut slots)?;

    let Some(slot) = slots.iter().find(|slot| slot.key == key) else {
        return Ok(None);
    };

    Ok(Some(ext_from_vox_value(&slot.value)?))
}
