use crate::MVoxExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its MagicaVoxel provenance as its ext, the state
/// the converters exchange.
pub type MVoxVoxMain = VoxMain<MVoxExt>;
