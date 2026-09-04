use crate::QbExt;
use voxcore::VoxMain;

/// The state the Qubicle qb converters exchange.
pub type QbVoxMain = VoxMain<Option<QbExt>>;
