use crate::ext::MVoxExt;
use voxcore::VoxMain;

/// The state the typed path exchanges: a [`VoxMain`] carrying its MagicaVoxel
/// provenance as its ext, or `None` for a state with none.
pub type MVoxVoxMain = VoxMain<Option<MVoxExt>>;
