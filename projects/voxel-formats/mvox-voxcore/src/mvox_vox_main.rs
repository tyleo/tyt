use crate::MVoxExt;
use voxcore::VoxMain;

/// The state the MagicaVoxel converters exchange.
pub type MVoxVoxMain = VoxMain<Option<MVoxExt>>;
