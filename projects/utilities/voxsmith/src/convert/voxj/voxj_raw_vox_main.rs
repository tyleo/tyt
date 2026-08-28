use crate::VoxjRawExt;
use voxcore::VoxMain;

/// The state a verbatim Voxel Json re-encode exchanges.
pub type VoxjRawVoxMain = VoxMain<Option<VoxjRawExt>>;
