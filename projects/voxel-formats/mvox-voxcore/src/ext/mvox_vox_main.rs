use crate::ext::MVoxExt;
use voxcore::VoxMain;

/// The state the typed path exchanges: a [`VoxMain`] carrying its MagicaVoxel
/// provenance as its ext.
pub type MVoxVoxMain = VoxMain<MVoxExt>;
