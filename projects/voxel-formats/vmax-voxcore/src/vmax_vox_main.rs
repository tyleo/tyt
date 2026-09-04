use crate::VMaxExt;
use voxcore::VoxMain;

/// The state the Voxel Max converters exchange.
pub type VMaxVoxMain = VoxMain<Option<VMaxExt>>;
