use crate::ext::QbExt;
use voxcore::VoxMain;

/// The state the Qubicle Binary typed path exchanges: a [`VoxMain`] carrying
/// its `.qb` provenance as its ext, or `None` for a state with none.
pub type QbVoxMain = VoxMain<Option<QbExt>>;
