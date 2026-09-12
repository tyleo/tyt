use crate::QbtExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its Qubicle Binary Tree provenance as its ext, the
/// state the `.qbt` converters exchange.
pub type QbtVoxMain = VoxMain<QbtExt>;
