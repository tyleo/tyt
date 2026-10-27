use crate::ForwardDependencies;
use qbcl_voxcore::codec::dependencies::{CompressZlib, DecompressZlib};

/// The Qubicle codec's dependencies a caller supplies.
pub trait QbclDependencies {
    /// The Qubicle codec's dependencies.
    type Qbcl: CompressZlib + DecompressZlib;

    /// The Qubicle codec's dependencies.
    fn qbcl(&self) -> &Self::Qbcl;
}

/// Forwards to the target's.
impl<T: ForwardDependencies<Target: QbclDependencies>> QbclDependencies for T {
    type Qbcl = <T::Target as QbclDependencies>::Qbcl;

    fn qbcl(&self) -> &Self::Qbcl {
        self.target().qbcl()
    }
}
