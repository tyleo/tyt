use crate::ext::QbtExt;
use voxcore::VoxMain;

/// The state the Qubicle Binary Tree typed path exchanges: a [`VoxMain`]
/// carrying its `.qbt` provenance as its ext.
pub type QbtVoxMain = VoxMain<QbtExt>;
