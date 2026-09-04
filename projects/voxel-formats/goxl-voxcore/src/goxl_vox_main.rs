use crate::GoxlExt;
use voxcore::VoxMain;

/// The state the Goxel converters exchange.
pub type GoxlVoxMain = VoxMain<Option<GoxlExt>>;
