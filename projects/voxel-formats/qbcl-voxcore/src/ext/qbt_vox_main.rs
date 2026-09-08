use crate::ext::QbtExt;
use voxcore::VoxMain;

/// The state the Qubicle Binary Tree typed path exchanges: a [`VoxMain`]
/// carrying its `.qbt` provenance as its ext, or `None` for a state with
/// none.
pub type QbtVoxMain = VoxMain<Option<QbtExt>>;
