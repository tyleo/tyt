use crate::GoxelExt;
use voxcore::VoxMain;

/// The state the Goxel converters exchange.
pub type GoxelVoxMain = VoxMain<Option<GoxelExt>>;
