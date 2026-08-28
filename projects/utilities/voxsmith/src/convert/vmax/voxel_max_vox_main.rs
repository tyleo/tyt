use crate::VoxelMaxExt;
use voxcore::VoxMain;

/// The state the Voxel Max converters exchange.
pub type VoxelMaxVoxMain = VoxMain<Option<VoxelMaxExt>>;
