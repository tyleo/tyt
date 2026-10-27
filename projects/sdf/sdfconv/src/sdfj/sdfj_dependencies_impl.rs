use crate::{DependenciesImpl, sdfj::SdfjDependencies};
use sdfj_sdfcore::codec::dependencies::DependenciesImpl as SdfjDependenciesImpl;

impl SdfjDependencies for DependenciesImpl {
    type Sdfj = SdfjDependenciesImpl;

    fn sdfj(&self) -> &Self::Sdfj {
        &SdfjDependenciesImpl
    }
}
