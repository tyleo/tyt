use crate::QbtExt;
use voxcore::VoxMain;

/// The state the Qubicle qbt converters exchange.
pub type QbtVoxMain = VoxMain<Option<QbtExt>>;
