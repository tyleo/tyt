use crate::Dependencies;
use ty_fbx::DependenciesImpl as TyFbxDependenciesImpl;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    type TyFbxDependencies = TyFbxDependenciesImpl;

    fn ty_fbx_dependencies(&self) -> Self::TyFbxDependencies {
        TyFbxDependenciesImpl
    }
}
