use crate::{DependenciesImpl, voxj::VoxjDependencies};
use voxj_voxcore::codec::dependencies::DependenciesImpl as VoxjDependenciesImpl;

impl VoxjDependencies for DependenciesImpl {
    type Voxj = VoxjDependenciesImpl;

    fn voxj(&self) -> &Self::Voxj {
        &VoxjDependenciesImpl
    }
}
