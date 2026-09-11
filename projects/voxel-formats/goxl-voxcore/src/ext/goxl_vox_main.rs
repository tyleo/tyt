use crate::ext::GoxlExt;
use voxcore::VoxMain;

/// The state the typed path exchanges: a [`VoxMain`] carrying its Goxel
/// provenance as its ext.
pub type GoxlVoxMain = VoxMain<GoxlExt>;
