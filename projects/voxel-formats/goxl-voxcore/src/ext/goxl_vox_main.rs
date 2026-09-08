use crate::ext::GoxlExt;
use voxcore::VoxMain;

/// The state the typed path exchanges: a [`VoxMain`] carrying its Goxel
/// provenance as its ext, or `None` for a state with none.
pub type GoxlVoxMain = VoxMain<Option<GoxlExt>>;
