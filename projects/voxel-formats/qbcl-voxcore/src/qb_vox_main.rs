use crate::QbExt;
use voxcore::VoxMain;

/// A [`VoxMain`] carrying its Qubicle Binary provenance as its ext, the
/// state the `.qb` converters exchange.
pub type QbVoxMain = VoxMain<QbExt>;
