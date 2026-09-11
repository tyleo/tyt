use crate::ext::QbExt;
use voxcore::VoxMain;

/// The state the Qubicle Binary typed path exchanges: a [`VoxMain`] carrying
/// its `.qb` provenance as its ext.
pub type QbVoxMain = VoxMain<QbExt>;
