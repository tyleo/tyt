use crate::{DependenciesImpl, vmax::VMaxDependencies};
use vmax_voxcore::codec::dependencies::DependenciesImpl as VMaxDependenciesImpl;

impl VMaxDependencies for DependenciesImpl {
    type VMax = VMaxDependenciesImpl;

    fn vmax(&self) -> &Self::VMax {
        &VMaxDependenciesImpl
    }
}
