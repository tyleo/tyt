use crate::QubicleQbclExt;
use voxcore::VoxMain;

/// The state the Qubicle qbcl converters exchange.
pub type QubicleQbclVoxMain = VoxMain<Option<QubicleQbclExt>>;
