use crate::QbclExt;
use voxcore::VoxMain;

/// The state the Qubicle qbcl converters exchange.
pub type QbclVoxMain = VoxMain<Option<QbclExt>>;
