use crate::MagicaVoxelExt;
use voxcore::VoxMain;

/// The state the MagicaVoxel converters exchange.
pub type MagicaVoxelVoxMain = VoxMain<Option<MagicaVoxelExt>>;
