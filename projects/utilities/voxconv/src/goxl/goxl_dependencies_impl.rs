use crate::{DependenciesImpl, goxl::GoxlDependencies};
use goxl_voxcore::codec::dependencies::DependenciesImpl as GoxlDependenciesImpl;

impl GoxlDependencies for DependenciesImpl {
    type Goxl = GoxlDependenciesImpl;

    fn goxl(&self) -> &Self::Goxl {
        &GoxlDependenciesImpl
    }
}
