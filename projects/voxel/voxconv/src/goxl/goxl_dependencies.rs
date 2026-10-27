use crate::ForwardDependencies;
use goxl_voxcore::codec::dependencies::{DecodePng, EncodePng};

/// The Goxel codec's dependencies a caller supplies.
pub trait GoxlDependencies {
    /// The Goxel codec's dependencies.
    type Goxl: DecodePng + EncodePng;

    /// The Goxel codec's dependencies.
    fn goxl(&self) -> &Self::Goxl;
}

/// Forwards to the target's.
impl<T: ForwardDependencies<Target: GoxlDependencies>> GoxlDependencies for T {
    type Goxl = <T::Target as GoxlDependencies>::Goxl;

    fn goxl(&self) -> &Self::Goxl {
        self.target().goxl()
    }
}
