use crate::GoxlExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its Goxel provenance as its ext, the state the
/// converters exchange.
pub type GoxlVoxMain = VoxMain<GoxlExt>;
