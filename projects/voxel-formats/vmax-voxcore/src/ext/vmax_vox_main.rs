use crate::ext::VMaxExt;
use voxcore::VoxMain;

/// The state the typed path exchanges: a [`VoxMain`] carrying its Voxel Max
/// provenance as its ext.
pub type VMaxVoxMain = VoxMain<VMaxExt>;
