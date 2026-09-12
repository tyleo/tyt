use crate::VoxjVoxExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its `ext` block as it was parsed, the main the
/// converters exchange.
pub type VoxjVoxMain = VoxMain<VoxjVoxExt>;
