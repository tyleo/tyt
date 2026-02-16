use crate::Dependencies;
use tyt_fbx::DependenciesImpl as TytFbxDependenciesImpl;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    type TytFbxDependencies = TytFbxDependenciesImpl;

    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies {
        TytFbxDependenciesImpl
    }
}
