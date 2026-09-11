use crate::{ext::VoxjVoxExt, voxj_map_from_vox_map_entries};
use voxcore::VoxExt;
use voxj::VoxjMap;

/// What a writer takes for the `ext` block. [`VoxjVoxExt`] writes its slots.
/// `()` writes no block.
pub trait VoxjVoxExtSource: VoxExt {
    /// The `ext` block to write, or `None` for no block. An empty ext is no
    /// block.
    fn ext_block(&self) -> Option<VoxjMap>;
}

impl VoxjVoxExtSource for VoxjVoxExt {
    fn ext_block(&self) -> Option<VoxjMap> {
        (!self.is_empty()).then(|| voxj_map_from_vox_map_entries(self.slots()))
    }
}

impl VoxjVoxExtSource for () {
    fn ext_block(&self) -> Option<VoxjMap> {
        None
    }
}
