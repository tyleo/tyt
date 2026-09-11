use crate::ext::VoxjVoxExt;
use voxcore::VoxMain;

/// The state the typed path exchanges: a [`VoxMain`] carrying its `ext`
/// block as it was parsed.
pub type VoxjVoxMain = VoxMain<VoxjVoxExt>;
