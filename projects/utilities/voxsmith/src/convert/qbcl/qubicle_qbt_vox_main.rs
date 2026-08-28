use crate::QubicleQbtExt;
use voxcore::VoxMain;

/// The state the Qubicle qbt converters exchange.
pub type QubicleQbtVoxMain = VoxMain<Option<QubicleQbtExt>>;
