use crate::{DependenciesImpl, qbcl::QbclDependencies};
use qbcl_voxcore::codec::dependencies::DependenciesImpl as QbclDependenciesImpl;

impl QbclDependencies for DependenciesImpl {
    type Qbcl = QbclDependenciesImpl;

    fn qbcl(&self) -> &Self::Qbcl {
        &QbclDependenciesImpl
    }
}
