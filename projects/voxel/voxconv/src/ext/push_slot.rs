use crate::{Error, Result};
use voxcore::VoxMapEntry;

/// Appends one slot. Errors when the block already holds its key.
pub fn push_slot(slots: &mut Vec<VoxMapEntry>, slot: VoxMapEntry) -> Result<()> {
    if slots.iter().any(|taken| taken.key == slot.key) {
        return Err(Error::Ext(format!(
            "the ext block holds `{}` twice",
            slot.key
        )));
    }

    slots.push(slot);

    Ok(())
}
