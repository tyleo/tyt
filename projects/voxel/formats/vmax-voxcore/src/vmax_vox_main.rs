use crate::VMaxExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its Voxel Max provenance as its ext, the state the
/// converters exchange.
pub type VMaxVoxMain = VoxMain<VMaxExt>;
