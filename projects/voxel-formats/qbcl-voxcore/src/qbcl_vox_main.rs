use crate::QbclExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its Qubicle Construction Library provenance as its
/// ext, the state the `.qbcl` converters exchange.
pub type QbclVoxMain = VoxMain<QbclExt>;
