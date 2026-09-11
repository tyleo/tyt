use crate::ext::QbclExt;
use voxcore::VoxMain;

/// The state the Qubicle Construction Library typed path exchanges: a
/// [`VoxMain`] carrying its `.qbcl` provenance as its ext.
pub type QbclVoxMain = VoxMain<QbclExt>;
