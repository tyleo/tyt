use crate::{Result, ext::VoxjVoxExt, vox_map_from_voxj_map};
use voxcore::{VoxExt, VoxMain};
use voxj::VoxjMap;

/// Puts the ext a read yields on the bare state. [`VoxjVoxExt`] keeps the
/// `ext` block. `()` drops it.
pub trait VoxjVoxExtSink: VoxExt + Sized {
    /// Puts on `state` the ext of a document whose `ext` block is `block`.
    /// Errors if the block holds a non-finite number or a repeated key.
    fn record_block(state: VoxMain<()>, block: Option<&VoxjMap>) -> Result<VoxMain<Self>>;
}

impl VoxjVoxExtSink for VoxjVoxExt {
    fn record_block(state: VoxMain<()>, block: Option<&VoxjMap>) -> Result<VoxMain<Self>> {
        let slots = match block {
            Some(block) => vox_map_from_voxj_map(block)?.into_entries(),
            None => Vec::new(),
        };

        Ok(state.put_ext(VoxjVoxExt::new(slots)))
    }
}

impl VoxjVoxExtSink for () {
    fn record_block(state: VoxMain<()>, _block: Option<&VoxjMap>) -> Result<VoxMain<Self>> {
        Ok(state)
    }
}
