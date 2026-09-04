use crate::QubicleQbExt;
use voxcore::VoxMain;

/// The state the Qubicle qb converters exchange.
pub type QubicleQbVoxMain = VoxMain<Option<QubicleQbExt>>;
